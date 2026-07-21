pub(super) fn version_satisfies(version: &str, requirement: &str) -> Result<bool, String> {
    let version =
        semver::Version::parse(version).map_err(|error| format!("invalid version: {error}"))?;
    let requirement = semver::VersionReq::parse(requirement)
        .map_err(|error| format!("invalid version requirement: {error}"))?;
    Ok(requirement.matches(&version))
}
