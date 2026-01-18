# Guidelines for AI Agents

## Commenting Strategy

Distinguish between comments intended for the user in the chat and comments left
in the codebase.

- **User Explanations:** Feel free to include miscellaneous details when
  explaining things to the user in the chat.
- **Codebase Comments:** Ensure comments added to the code are necessary and
  beneficial for future readers.
  - **Avoid the Obvious:** Do not comment on things that are immediately clear
    from reading the code.
  - **Complex Logic:** Provide concise explanations for complex code sections.
  - **Context & Rationale:** Even for simple code, explain the "why" if it's not
    self-evident (e.g., purpose, usage).
  - **Design Decisions:** When relevant, document why a specific approach was
    chosen over alternatives ("why not").

## Communication & Clarification

Do not hesitate to ask the user when facing ambiguities, confusing code, or
issues where user input is the best solution.

- **Self-Resolution First:** Make a meaningful attempt to resolve the issue
  independently using available tools before asking.
- **Efficiency:** If solving the problem independently would be excessively
  complex, prioritize asking the user.

## Code Quality

- **Proactive Verification:** Whenever tools or commands are available, verify
  code changes yourself to ensure correctness and code quality.
- **Automated Checks:** Always execute formatters and linters to enforce coding
  standards and catch potential issues early. (Use `cargo clippy --all-targets`
  to ensure test code is also checked.)
- **Refinement & Cleanup:** Review changes to ensure they are minimal and
  sufficient. Check for and remove any unnecessary modifications, unused
  variables, or leftover dependencies.
