#[cfg(test)]
mod tests;

use smol_str::SmolStr;
use std::path::Path;
use vec1::vec1;

use crate::{
    ast::{Definition, Function, Statement, TargettedDefinition, UntypedExpr, UntypedModule},
    build::Target,
    format::{Formatter, Intermediate},
    Error, Result,
};

pub fn parse_fix_and_format(src: &SmolStr, path: &Path) -> Result<String> {
    // Parse
    let parsed = crate::parse::parse_module(src).map_err(|error| Error::Parse {
        path: path.to_path_buf(),
        src: src.clone(),
        error,
    })?;
    let mut module = parsed.module;
    let intermediate = Intermediate::from_extra(&parsed.extra, src);

    // Fix
    Fixer::fix(&mut module);

    // Format
    let mut buffer = String::new();
    Formatter::with_comments(&intermediate)
        .module(&module)
        .pretty_print(80, &mut buffer)?;

    Ok(buffer)
}

#[derive(Debug, Clone, Copy)]
pub struct Fixer {}

impl Fixer {
    pub fn fix(module: &mut UntypedModule) {
        Self {}.fix_module(module)
    }

    fn fix_module(&mut self, module: &mut UntypedModule) {
        for definition in module.definitions.iter_mut() {
            self.fix_definition(definition);
        }
    }

    fn fix_definition(&mut self, definition: &mut TargettedDefinition) {
        match definition {
            TargettedDefinition::Any(Definition::ExternalFunction(external_function)) => {
                let function = self.convert_function(None, external_function);
                let new = TargettedDefinition::Any(Definition::Function(function));
                drop(std::mem::replace(definition, new));
            }

            TargettedDefinition::Only(target, Definition::ExternalFunction(external_function)) => {
                let function = self.convert_function(Some(*target), external_function);
                let new = TargettedDefinition::Any(Definition::Function(function));
                drop(std::mem::replace(definition, new));
            }

            _ => (),
        }
    }

    fn convert_function(
        &mut self,
        target: Option<Target>,
        external_function: &mut crate::ast::ExternalFunction<()>,
    ) -> Function<(), UntypedExpr> {
        let mut function = Function {
            location: external_function.location,
            end_position: external_function.location.end,
            name: external_function.name.clone(),
            body: vec1![Statement::Expression(UntypedExpr::Placeholder {
                location: external_function.location,
            })],
            public: external_function.public,
            return_annotation: Some(external_function.return_.clone()),
            return_type: (),
            documentation: None,
            external_erlang: None,
            external_javascript: None,
            // TODO: arguments
            arguments: vec![],
        };

        let external = Some((
            external_function.module.clone(),
            external_function.fun.clone(),
        ));
        match self.external_target(target, external_function) {
            Some(Target::Erlang) => function.external_erlang = external,
            Some(Target::JavaScript) => function.external_javascript = external,
            None => todo!("Handle unknown"),
        }
        function
    }

    fn external_target(
        &self,
        target: Option<Target>,
        external_function: &mut crate::ast::ExternalFunction<()>,
    ) -> Option<Target> {
        if let Some(target) = target {
            Some(target)
        } else if external_function.module.ends_with(".jsx") {
            Some(Target::JavaScript)
        } else if external_function.module.ends_with(".js") {
            Some(Target::JavaScript)
        } else if external_function.module.ends_with(".tsx") {
            Some(Target::JavaScript)
        } else if external_function.module.ends_with(".ts") {
            Some(Target::JavaScript)
        } else if external_function.module.ends_with(".mjs") {
            Some(Target::JavaScript)
        } else if external_function.module.contains("/") {
            Some(Target::JavaScript)
        } else if external_function.module.starts_with("Elixir.") {
            Some(Target::Erlang)
        } else {
            None
        }
    }
}