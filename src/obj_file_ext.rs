use target_lexicon::{Triple, OperatingSystem, Environment};

pub fn object_file_extension(triple: &Triple) -> &'static str {
    match (triple.operating_system, triple.environment) {
        (OperatingSystem::Windows | OperatingSystem::Uefi, _)
        | (_, Environment::Msvc) => "obj",
        _ => "o"
    }
}
