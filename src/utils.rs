use std::{path::{Path, PathBuf}, fs::File, io::Read};

pub fn read_at_path<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let mut f = File::open(path)?;
    let mut code: String = String::new();
    f.read_to_string(&mut code)?;

    Ok(code)
}

pub fn make_relative_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, std::io::Error> {
    match path
        .as_ref()
        .canonicalize()?
        .strip_prefix(std::env::current_dir()?.canonicalize()?)
    {
        Ok(relative_path) => Ok(replace_slashes(relative_path)),
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "unable to strip prefix!",
        )),
    }
}

// back slashes to forward slashes
pub fn replace_slashes<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut os_str = std::ffi::OsString::new();
    let count = path.as_ref().components().count();
    for (i, component) in path.as_ref().components().enumerate() {
        os_str.push(component.as_os_str());
        if i != count - 1 {
            os_str.push("/");
        }
    }
    PathBuf::from(os_str)
}