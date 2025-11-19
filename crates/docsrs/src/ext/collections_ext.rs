/// Extension trait for formatting slices of GenericBound
pub trait GenericBoundsExt {
    fn format_bounds(&self) -> String;
}

impl GenericBoundsExt for [rustdoc_types::GenericBound] {
    fn format_bounds(&self) -> String {
        self.iter()
            .map(|bound| match bound {
                rustdoc_types::GenericBound::TraitBound { trait_, .. } => trait_.path.clone(),
                rustdoc_types::GenericBound::Outlives(lifetime) => lifetime.clone(),
                rustdoc_types::GenericBound::Use(args) => {
                    // Format precise capturing args
                    let arg_strs: Vec<String> = args
                        .iter()
                        .map(|arg| match arg {
                            rustdoc_types::PreciseCapturingArg::Lifetime(lt) => lt.clone(),
                            rustdoc_types::PreciseCapturingArg::Param(name) => name.clone(),
                        })
                        .collect();
                    format!("use<{}>", arg_strs.join(", "))
                }
            })
            .collect::<Vec<_>>()
            .join(" + ")
    }
}
