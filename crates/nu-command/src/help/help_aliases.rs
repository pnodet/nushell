use crate::filters::find_internal;
use nu_engine::{command_prelude::*, scope::ScopeData};

#[derive(Clone)]
pub struct HelpAliases;

impl Command for HelpAliases {
    fn name(&self) -> &str {
        "help aliases"
    }

    fn description(&self) -> &str {
        "Show help on nushell aliases."
    }

    fn signature(&self) -> Signature {
        Signature::build("help aliases")
            .category(Category::Core)
            .rest(
                "rest",
                SyntaxShape::String,
                "The name of alias to get help on.",
            )
            .named(
                "find",
                SyntaxShape::String,
                "string to find in alias names and descriptions",
                Some('f'),
            )
            .input_output_types(vec![(Type::Nothing, Type::table())])
            .allow_variants_without_examples(true)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "show all aliases",
                example: "help aliases",
                result: None,
            },
            Example {
                description: "show help for single alias",
                example: "help aliases my-alias",
                result: None,
            },
            Example {
                description: "search for string in alias names and descriptions",
                example: "help aliases --find my-alias",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        help_aliases(engine_state, stack, call)
    }
}

pub fn help_aliases(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<PipelineData, ShellError> {
    let head = call.head;
    let find: Option<Spanned<String>> = call.get_flag(engine_state, stack, "find")?;
    let rest: Vec<Spanned<String>> = call.rest(engine_state, stack, 0)?;

    if let Some(f) = find {
        let all_cmds_vec = build_help_aliases(engine_state, stack, head);
        return find_internal(
            all_cmds_vec,
            engine_state,
            stack,
            &f.item,
            &["name", "description"],
            true,
        );
    }

    if rest.is_empty() {
        Ok(build_help_aliases(engine_state, stack, head))
    } else {
        let mut name = String::new();

        for r in &rest {
            if !name.is_empty() {
                name.push(' ');
            }
            name.push_str(&r.item);
        }

        let Some(alias) = engine_state.find_decl(name.as_bytes(), &[]) else {
            return Err(ShellError::AliasNotFound {
                span: Span::merge_many(rest.iter().map(|s| s.span)),
            });
        };

        let Some(alias) = engine_state.get_decl(alias).as_alias() else {
            return Err(ShellError::AliasNotFound {
                span: Span::merge_many(rest.iter().map(|s| s.span)),
            });
        };

        let alias_expansion =
            String::from_utf8_lossy(engine_state.get_span_contents(alias.wrapped_call.span));
        let description = alias.description();
        let extra_desc = alias.extra_description();

        // TODO: merge this into documentation.rs at some point
        const G: &str = "\x1b[32m"; // green
        const C: &str = "\x1b[36m"; // cyan
        const RESET: &str = "\x1b[0m"; // reset

        let mut long_desc = String::new();

        long_desc.push_str(description);
        long_desc.push_str("\n\n");

        if !extra_desc.is_empty() {
            long_desc.push_str(extra_desc);
            long_desc.push_str("\n\n");
        }

        long_desc.push_str(&format!("{G}Alias{RESET}: {C}{name}{RESET}"));
        long_desc.push_str("\n\n");
        long_desc.push_str(&format!("{G}Expansion{RESET}:\n  {alias_expansion}"));

        let config = stack.get_config(engine_state);
        if !config.use_ansi_coloring.get(engine_state) {
            long_desc = nu_utils::strip_ansi_string_likely(long_desc);
        }

        Ok(Value::string(long_desc, call.head).into_pipeline_data())
    }
}

fn build_help_aliases(engine_state: &EngineState, stack: &Stack, span: Span) -> PipelineData {
    let mut scope_data = ScopeData::new(engine_state, stack);
    scope_data.populate_decls();

    Value::list(scope_data.collect_aliases(span), span).into_pipeline_data()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_examples() {
        use super::HelpAliases;
        use crate::test_examples;
        test_examples(HelpAliases {})
    }
}
