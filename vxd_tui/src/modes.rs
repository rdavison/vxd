//! Mode management implementation.
//!
//! This module provides a concrete implementation of Vim's mode system.

use vxd::modes::{
    is_valid_transition, CtrlOMode, InvalidTransition, Mode, ModeManager, ModeState,
    ModeTransition,
};

/// Concrete mode manager implementation
#[derive(Debug, Clone)]
pub struct TuiModeManager {
    state: ModeState,
}

impl TuiModeManager {
    /// Create a new mode manager starting in normal mode
    pub fn new() -> Self {
        TuiModeManager {
            state: ModeState::new(),
        }
    }
}

impl Default for TuiModeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeManager for TuiModeManager {
    fn mode(&self) -> Mode {
        self.state.mode
    }

    fn state(&self) -> &ModeState {
        &self.state
    }

    fn can_transition_to(&self, target: Mode) -> bool {
        is_valid_transition(self.state.mode, target)
    }

    fn transition_to(&mut self, target: Mode) -> Result<ModeTransition, InvalidTransition> {
        let from = self.state.mode;

        if !is_valid_transition(from, target) {
            return Err(InvalidTransition {
                from,
                to: target,
                reason: format!("Cannot transition from {:?} to {:?}", from, target),
            });
        }

        // Clear ctrl_o mode when transitioning to normal
        if target == Mode::Normal {
            self.state.ctrl_o_mode = None;
        }

        // Clear pending operator when leaving operator-pending
        if from == Mode::OperatorPending {
            self.state.pending_operator = None;
            self.state.blocking = false;
        }

        self.state.mode = target;

        Ok(ModeTransition {
            from,
            to: target,
            trigger: None,
        })
    }

    fn enter_operator_pending(
        &mut self,
        operator: char,
    ) -> Result<ModeTransition, InvalidTransition> {
        let from = self.state.mode;

        if from != Mode::Normal && !matches!(from, Mode::Visual(_)) {
            return Err(InvalidTransition {
                from,
                to: Mode::OperatorPending,
                reason: "Can only enter operator-pending from normal or visual mode".to_string(),
            });
        }

        self.state.pending_operator = Some(operator);
        self.state.blocking = true;
        self.state.mode = Mode::OperatorPending;

        Ok(ModeTransition {
            from,
            to: Mode::OperatorPending,
            trigger: Some(operator.to_string()),
        })
    }

    fn exit_operator_pending(&mut self) -> Result<ModeTransition, InvalidTransition> {
        if self.state.mode != Mode::OperatorPending {
            return Err(InvalidTransition {
                from: self.state.mode,
                to: Mode::Normal,
                reason: "Not in operator-pending mode".to_string(),
            });
        }

        self.state.pending_operator = None;
        self.state.blocking = false;
        self.state.mode = Mode::Normal;

        Ok(ModeTransition {
            from: Mode::OperatorPending,
            to: Mode::Normal,
            trigger: None,
        })
    }

    fn enter_ctrl_o(&mut self) -> Result<(), InvalidTransition> {
        match self.state.mode {
            Mode::Insert => {
                self.state.ctrl_o_mode = Some(CtrlOMode::FromInsert);
                Ok(())
            }
            Mode::Visual(_) | Mode::Select(_) => {
                self.state.ctrl_o_mode = Some(CtrlOMode::FromVisual);
                Ok(())
            }
            _ => Err(InvalidTransition {
                from: self.state.mode,
                to: self.state.mode, // Not really a transition
                reason: "Ctrl-O only available from insert or visual mode".to_string(),
            }),
        }
    }

    fn exit_ctrl_o(&mut self) -> Result<(), InvalidTransition> {
        match self.state.ctrl_o_mode {
            Some(CtrlOMode::FromInsert) => {
                self.state.ctrl_o_mode = None;
                Ok(())
            }
            Some(CtrlOMode::FromVisual) => {
                self.state.ctrl_o_mode = None;
                Ok(())
            }
            None => Err(InvalidTransition {
                from: self.state.mode,
                to: self.state.mode,
                reason: "Not in Ctrl-O mode".to_string(),
            }),
        }
    }

    fn set_blocking(&mut self, blocking: bool) {
        self.state.blocking = blocking;
    }

    fn set_count(&mut self, count: Option<usize>) {
        self.state.count = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vxd::modes::VisualMode;

    #[test]
    fn test_normal_to_insert() {
        let mut mgr = TuiModeManager::new();
        assert_eq!(mgr.mode(), Mode::Normal);

        let result = mgr.enter_insert();
        assert!(result.is_ok());
        assert_eq!(mgr.mode(), Mode::Insert);
    }

    #[test]
    fn test_insert_to_normal() {
        let mut mgr = TuiModeManager::new();
        mgr.enter_insert().ok();
        assert_eq!(mgr.mode(), Mode::Insert);

        let result = mgr.escape_to_normal();
        assert!(result.is_ok());
        assert_eq!(mgr.mode(), Mode::Normal);
    }

    #[test]
    fn test_visual_mode_switching() {
        let mut mgr = TuiModeManager::new();

        mgr.enter_visual().ok();
        assert_eq!(mgr.mode(), Mode::Visual(VisualMode::Char));

        mgr.transition_to(Mode::Visual(VisualMode::Line)).ok();
        assert_eq!(mgr.mode(), Mode::Visual(VisualMode::Line));

        mgr.transition_to(Mode::Visual(VisualMode::Block)).ok();
        assert_eq!(mgr.mode(), Mode::Visual(VisualMode::Block));
    }

    #[test]
    fn test_operator_pending_blocking() {
        let mut mgr = TuiModeManager::new();
        mgr.enter_operator_pending('d').ok();

        assert!(mgr.state().is_blocking());
        assert_eq!(mgr.state().pending_operator, Some('d'));
    }

    #[test]
    fn test_ctrl_o_from_insert() {
        let mut mgr = TuiModeManager::new();
        mgr.enter_insert().ok();
        mgr.enter_ctrl_o().ok();

        assert_eq!(mgr.state().effective_code(), "niI");
    }

    #[test]
    fn test_mode_codes() {
        let mut mgr = TuiModeManager::new();

        assert_eq!(mgr.mode().code(), "n");

        mgr.enter_insert().ok();
        assert_eq!(mgr.mode().code(), "i");

        mgr.escape_to_normal().ok();
        mgr.enter_replace().ok();
        assert_eq!(mgr.mode().code(), "R");

        mgr.escape_to_normal().ok();
        mgr.enter_visual().ok();
        assert_eq!(mgr.mode().code(), "v");

        mgr.transition_to(Mode::Visual(VisualMode::Line)).ok();
        assert_eq!(mgr.mode().code(), "V");
    }

    #[test]
    fn test_mode_display_names() {
        let mut mgr = TuiModeManager::new();

        assert_eq!(mgr.mode().display_name(), "");

        mgr.enter_insert().ok();
        assert_eq!(mgr.mode().display_name(), "-- INSERT --");

        mgr.escape_to_normal().ok();
        mgr.enter_visual_line().ok();
        assert_eq!(mgr.mode().display_name(), "-- VISUAL LINE --");
    }
}
