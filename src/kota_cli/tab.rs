use colored::*;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Context, Helper};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashSet;

pub struct KotaHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    commands: HashSet<String>,
}

impl Default for KotaHelper {
    fn default() -> Self {
        let mut commands = HashSet::new();
        commands.insert("/quit".to_string());
        commands.insert("/exit".to_string());
        commands.insert("/config".to_string());
        commands.insert("/help".to_string());
        commands.insert("/history".to_string());
        commands.insert("/skills".to_string());
        commands.insert("/skill".to_string());
        commands.insert("/skill-off".to_string());
        commands.insert("/load".to_string());
        commands.insert("/sessions".to_string());
        commands.insert("/delete".to_string());

        Self {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            commands,
        }
    }
}

impl Completer for KotaHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        if line.starts_with('/') {
            let input = &line[..pos];
            let mut matches = Vec::new();

            for command in &self.commands {
                if command.starts_with(input) {
                    matches.push(Pair {
                        display: command.clone(),
                        replacement: command.clone(),
                    });
                }
            }

            // 按字母顺序排序
            matches.sort_by(|a, b| a.display.cmp(&b.display));

            Ok((0, matches))
        } else {
            Ok((pos, vec![]))
        }
    }
}

impl Hinter for KotaHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for KotaHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(prompt)
        } else {
            Owned(prompt.to_string())
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("{}", hint.dimmed()))
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        // 高亮显示命令
        if line.starts_with('/') {
            if let Some(space_pos) = line.find(' ') {
                let command = &line[..space_pos];
                let rest = &line[space_pos..];
                if self.commands.contains(command) {
                    return Owned(format!("{}{}", command.bright_green(), rest));
                }
            } else if self.commands.contains(line) {
                return Owned(line.bright_green().to_string());
            }
        }

        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for KotaHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Helper for KotaHelper {}
