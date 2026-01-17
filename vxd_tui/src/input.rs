use crate::editor::Editor;
use crate::key::{Key, parse_keys};
use vxd::mappings::{MappingCheckResult, MappingManager};

/// Handles input buffering and mapping expansion.
#[derive(Debug, Default)]
pub struct InputHandler {
    buffer: String,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        InputHandler {
            buffer: String::new(),
        }
    }

    /// Process a key and return a sequence of keys to be executed.
    pub fn handle_key(&mut self, key: Key, editor: &Editor) -> Vec<Key> {
        self.buffer.push_str(&key.to_string());
        self.process_buffer(editor)
    }

    fn process_buffer(&mut self, editor: &Editor) -> Vec<Key> {
        let mut output = Vec::new();
        let mode = editor.mode();

        // Safety limit to prevent infinite loops in case of weird buffer states
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100;

        loop {
            if self.buffer.is_empty() {
                break;
            }
            if iterations > MAX_ITERATIONS {
                // Failsafe: return the buffer as raw keys and clear it
                let keys = parse_keys(&self.buffer);
                output.extend(keys);
                self.buffer.clear();
                break;
            }
            iterations += 1;

            match editor.mappings.check(mode, &self.buffer) {
                MappingCheckResult::FullMatch(m) => {
                    self.buffer.clear();
                    let rhs = parse_keys(&m.rhs);
                    // For now, treat all as noremap (return keys to execute)
                    // TODO: Handle recursive mappings by feeding them back?
                    // But that requires changing the return type or interface.
                    output.extend(rhs);
                    break;
                },
                MappingCheckResult::PartialMatch => {
                    // Wait for more input
                    break;
                },
                MappingCheckResult::NoMatch => {
                    // Pop first key
                    let keys = parse_keys(&self.buffer);
                    if let Some(first) = keys.first() {
                         let k_str = first.to_string();
                         output.push(*first);
                         
                         // Advance buffer
                         // We must be careful about prefix matching if multiple keys map to same string prefix?
                         // Key::to_string() should be consistent.
                         if self.buffer.starts_with(&k_str) {
                             self.buffer = self.buffer[k_str.len()..].to_string();
                         } else {
                             // Fallback if string representation doesn't match perfectly
                             // (e.g. if buffer accumulation logic differed)
                             // This handles the case where we can't cleanly strip the key.
                             // In that case, we just clear to avoid stuck state.
                             self.buffer.clear();
                         }
                    } else {
                        self.buffer.clear();
                    }
                }
            }
        }
        output
    }
}
