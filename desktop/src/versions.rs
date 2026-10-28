pub fn newer(remote: &str, local: &str) -> Result<bool, String> {
    let remote = semver::Version::parse(remote).map_err(|_| "Version distante invalide.")?;
    let local = semver::Version::parse(local).map_err(|_| "Version locale invalide.")?;
    Ok(remote.cmp_precedence(&local).is_gt())
}
pub fn supported(minimum: &str) -> Result<bool, String> {
    Ok(!newer(minimum, env!("CARGO_PKG_VERSION"))?)
}
