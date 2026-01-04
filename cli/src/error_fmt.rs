// Copyright 2018-2025 the Deno authors. MIT license.

use std::error::Error;
use std::fmt::Write;

/// Formats an error chain with numbered lines, similar to Deno's error formatting.
///
/// This function traverses the error's source chain and formats each unique
/// error message with a numbered prefix (0:, 1:, 2:, etc.).
///
/// Note: Deno's original implementation (`format_deno_graph_error` in `libs/resolver/graph.rs`)
/// displays the first error message without a number, and only numbers the source chain
/// starting from 0. Our implementation numbers all errors including the first one.
/// Deno's version also handles multi-line error messages with additional indentation.
pub fn format_error_chain(error: &dyn Error) -> String {
    let mut message = String::new();
    let mut display_count = 0;

    // Start with the error itself
    let current_message = error.to_string();
    write!(&mut message, "\n    {}: {}", display_count, current_message).unwrap();
    let mut past_message = current_message;
    display_count += 1;

    // Then traverse the source chain
    let mut maybe_source = error.source();
    while let Some(source) = maybe_source {
        let current_message = source.to_string();
        maybe_source = source.source();

        if current_message != past_message {
            write!(&mut message, "\n    {}: {}", display_count, current_message).unwrap();
            past_message = current_message;
            display_count += 1;
        }

        // Limit depth to prevent infinite loops
        if display_count >= 8 {
            message.push_str("\n    ...");
            break;
        }
    }

    message
}
