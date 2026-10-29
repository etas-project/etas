use super::*;

pub(super) fn to_metadata(
    template: &crate::PackageRequestInstantiationMetadata,
) -> Result<etas_package_metadata::RequestInstantiationTemplate, PackageError> {
    Ok(etas_package_metadata::RequestInstantiationTemplate {
        actions: template
            .actions
            .iter()
            .map(|instance| {
                Ok(etas_package_metadata::ActionRequestTemplate {
                    action: effect_ref_to_metadata(&instance.action)?,
                    signature: instance
                        .signature
                        .as_ref()
                        .map(|signature| {
                            Ok::<_, PackageError>(etas_package_metadata::ActionRequestSignature {
                                params: signature
                                    .params
                                    .iter()
                                    .map(type_to_metadata)
                                    .collect::<Result<_, _>>()?,
                                output: type_to_metadata(&signature.output)?,
                            })
                        })
                        .transpose()?,
                    type_bindings: instance
                        .type_bindings
                        .as_ref()
                        .map(|bindings| {
                            bindings
                                .iter()
                                .map(|(name, ty)| {
                                    Ok(etas_package_metadata::ActionTypeBinding {
                                        parameter: name.clone(),
                                        ty: type_to_metadata(ty)?,
                                    })
                                })
                                .collect::<Result<_, PackageError>>()
                        })
                        .transpose()?,
                })
            })
            .collect::<Result<_, PackageError>>()?,
        parameters: template
            .parameters
            .iter()
            .map(|parameter| {
                Ok(etas_package_metadata::ParameterRequestTemplate {
                    parameter: parameter.parameter.clone(),
                    excluded: parameter
                        .excluded
                        .iter()
                        .map(effect_ref_to_metadata)
                        .collect::<Result<_, _>>()?,
                })
            })
            .collect::<Result<_, PackageError>>()?,
    })
}
