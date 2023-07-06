use std::io::Read;
use std::path::Path;
use cdb::CDBWriter;
use std::fs::File;
use std::fs;
use flate2::read::GzDecoder;
use tar::Archive;
use unicode_width::UnicodeWidthStr;

fn get_pacman_repos(db_dir: &Path) -> std::io::Result<Vec<String>> {
    let mut res: Vec<String> = Vec::new();
    for entry in fs::read_dir(db_dir)? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap();
        if let Some(name) = name.strip_suffix(".files") {
            res.push(name.to_string());
        }
    }
    Ok(res)
}

fn write_db(path: &Path, reponame: &str, cdb: &mut CDBWriter) -> std::io::Result<()> {
    let tgz = File::open(path).expect("Could not open pacman file database");
    let tar = GzDecoder::new(tgz);
    let mut archive = Archive::new(tar);

    for entry in archive.entries()? {
        let mut e = entry?;
        if !e.path()?.ends_with("files") { continue }
        let p = e.path()?.to_path_buf();
        let pkgname = p.to_str().unwrap().strip_suffix("/files").unwrap();
        let mut read_buf: Vec<u8> = Vec::new();
        e.read_to_end(&mut read_buf)?;
        let file_text = String::from_utf8(read_buf).unwrap();
        for line in file_text.lines().skip(1) {
            if line.ends_with('/') { continue }
            if !line.contains("/bin/") && !line.contains("/sbin/") { continue }
            let basename = line.rsplit_once('/').map(|x| x.1).unwrap_or(line);
            cdb.add(basename.as_bytes(), (reponame.to_string() + "/" + pkgname + "/" + line).as_bytes())?;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let cdb_path = "/var/lib/pkgfile-tiny.cdb";
    let pacman_db_path = Path::new("/var/lib/pacman/sync");
    if let Some(arg) = std::env::args().skip(1).next() {
        // lookup
        let cdb = cdb::CDB::open(cdb_path).expect("could not open database file");
        let mut results: Vec<(String, String)> = Vec::new();
        let mut col_width = 0;
        for row in cdb.find(arg.as_bytes()) {
            // PaRsInG
            let row = String::from_utf8(row.unwrap()).unwrap();
            let mut iter = row.splitn(3, '/');
            let repo = iter.next().unwrap();
            let pkg = iter.next().unwrap();
            let path = "/".to_owned() + iter.next().unwrap();
            let mut iter = pkg.rsplitn(3, "-");
            let pkgrel = iter.next().unwrap();
            let pkgver = iter.next().unwrap();
            let pkgname = iter.next().unwrap();
            let firstpart = format!("{repo}/{pkgname} {pkgver}-{pkgrel}");
            col_width = col_width.max(UnicodeWidthStr::width(firstpart.as_str()));
            results.push((firstpart, path));
        }
        for (col1, col2) in results {
            println!("{}{}   {}", col1, " ".repeat(col_width - UnicodeWidthStr::width(col1.as_str())), col2);
        }
    } else {
        println!("updating database...");
        let mut cdb = CDBWriter::create(cdb_path).expect("failed to create database file");
        for x in get_pacman_repos(pacman_db_path).unwrap() {
            write_db(pacman_db_path.join(x.clone() + ".files").as_path(), x.as_str(), &mut cdb)?;
        }
        cdb.finish()?;
    }

    Ok(())
}
