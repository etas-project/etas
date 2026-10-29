use super::*;

pub(super) fn from_metadata(
    template: etas_package_metadata::RequestInstantiationTemplate,
) -> Result<crate::PackageRequestInstantiationMetadata, PackageError> {
    Ok(crate::PackageRequestInstantiationMetadata {
        actions: template
            .actions
            .into_iter()
            .map(|instance| {
                Ok(crate::PackageActionRequestMetadata {
                    action: effect_ref_from_metadata(instance.action)?,
                    signature: instance
                        .signature
                        .map(|signature| {
                            Ok::<_, PackageError>(crate::PackageActionRequestSignatureMetadata {
                                params: signature
                                    .params
                                    .into_iter()
                                    .map(type_from_metadata)
                                    .collect::<Result<_, _>>()?,
                                output: type_from_metadata(signature.output)?,
                            })
                        })
                        .transpose()?,
                    type_bindings: instance
                        .type_bindings
                        .map(|bindings| {
                            bindings
                                .into_iter()
                                .map(|binding| {
                                    Ok((binding.parameter, type_from_metadata(binding.ty)?))
                                })
                                .collect::<Result<_, PackageError>>()
                        })
                        .transpose()?,
                })
            })
            .collect::<Result<_, PackageError>>()?,
        parameters: template
            .parameters
            .into_iter()
            .map(|parameter| {
                Ok(crate::PackageParameterRequestMetadata {
                    parameter: parameter.parameter,
                    excluded: parameter
                        .excluded
                        .into_iter()
                        .map(effect_ref_from_metadata)
                        .collect::<Result<_, _>>()?,
                })
            })
            .collect::<Result<_, PackageError>>()?,
    })
}
