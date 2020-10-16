use psf;

fn is_empty(data: &psf::Glyph<u8>) -> bool {
    for x in 0..data.width() {
        for y in 0..data.height() {
            assert!(data.get(x, y).is_some());
            if data.get(x, y).unwrap() != 0 {
                return false;
            }
        }
    }
    true
}

#[cfg(unix)]
#[test]
fn read_consolefonts() {
    use std::fs::read_dir;
    use std::path::Path;

    let consolefonts_dir = Path::new("/usr/share/consolefonts");
    if !consolefonts_dir.exists() || !consolefonts_dir.is_dir() {
        return;
    }
    for d in read_dir(&consolefonts_dir).unwrap() {
        if let Ok(entry) = d {
            #[cfg(not(feature = "unzip"))]
            if entry.path().extension().unwrap() == "gz" {
                continue;
            }
            let path = consolefonts_dir.join(&entry.path());
            println!("processed path: {:?}", &path);
            let font = psf::Font::new(&path);
            assert!(font.is_ok());
            let font = font.unwrap();
            let c = font.get_char('X');
            assert!(c.is_some());
            let c = c.unwrap();
            assert!(c.width() > 0);
            assert!(c.height() > 0);
            assert!(!is_empty(&c));
        }
    }
}
