use std::error::Error;
use std::fmt;
use std::fmt::{format, Display};
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use audiotags2::{Album, Tag};
use lrc::{Lyrics, TimeTag};
use mpris::PlayerFinder;
use rusqlite::Connection;
use rusqlite::fallible_iterator::FallibleIterator;
use xdir::{config, home};
use crate::SnowstormErrors::NoIndex;

#[derive(Debug, Default, Clone)]
pub struct OverlayMetadata {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub time: u64,
}
#[derive(Debug, Clone)]
enum SnowstormErrors {
    NoIndex,
}
impl Display for SnowstormErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for SnowstormErrors {}
#[derive(Debug)]
pub struct SongMetadata {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub lyrics_location: Option<String>,
}
pub fn get_metadata() -> Result<OverlayMetadata, Box<dyn Error>> {
    let player = PlayerFinder::new()?.find_active()?;
    let metadata = player.get_metadata()?;
    Ok(OverlayMetadata {
        name: metadata.title().unwrap_or("").to_string(),
        artist: metadata.artists().unwrap_or(vec![""])[0].to_string(),
        album: metadata.album_name().unwrap_or("").to_string(),
        time: player.get_position_in_microseconds().unwrap() * 1000,
    })
}

fn write_to_database(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let p = path.with_extension("lrc");
    if let Ok(tags) = Tag::new().read_from_path(&path) {
        let metadata = SongMetadata {
            name: tags.title().unwrap_or_default().to_string(),
            artist: "".to_string(),
            album: tags.album().unwrap_or_else(|| {Album::with_title("")}).title.to_string(),
            lyrics_location: Some(p.to_str().unwrap_or("").to_string()),
        };
        eprintln!("{:?}", p);
        if p.exists() {
            let mut tags = Tag::new().read_from_path(&path).expect(format!("Could not read tags from {:?}", path).as_str());
            get_database().execute(r"
        INSERT INTO songs
(name,
album,

lyrics)
      values (?1, ?2, ?3)
        ", (metadata.name,
            metadata.album,
metadata.lyrics_location.unwrap()
            )

            ).expect(format!("Could not write to database: {:?}", path).as_str());}
    }

    Ok(())

}

pub fn read_folder(location: PathBuf) -> Result<(), Box<dyn Error>> {
    for path in location.read_dir()?{
        if let Ok(path) = path {
            if path.file_type()?.is_file() {
                write_to_database(path.path())?;
            }
            else if path.file_type()?.is_dir() {
                read_folder(path.path())?;
            }
        }
    }
    Ok(())
}

pub fn get_lyric(path: String, time: i64) -> Result<String, Box<dyn Error>> {
    eprintln!("{path}aa");
    let path = Path::new(&path);
    let mut lyrics = String::new();
    File::open(path)?.read_to_string(&mut lyrics)?;
    let lyrics = Lyrics::from_str(&lyrics)?;
    let timed_lines = lyrics.get_timed_lines();
    let idx = lyrics.find_timed_line_index(TimeTag::new(time));
    if let   Some(idx)=idx {    return Ok(timed_lines.get(idx).unwrap().1.to_string());
    }
    else{Err(NoIndex.into())}
}

pub fn search_db(data: OverlayMetadata) -> Result<String, Box<dyn Error>> {
let r: String = get_database()
    .prepare("SELECT lyrics FROM songs WHERE name = :n AND album = :a")
    ?
    .query_map(&[(":n", &data.name),(":a", &data.album)], |row| {

    Ok(row.get(0)?)
})?.next().unwrap_or(Ok("".to_string()))?;
    Ok(format!("{:?}",r).to_string())

}
pub fn init() {
    let path = config().map(|path| path.join("snowstormosd")).unwrap_or_default().join("snowstormosd.sqlite");
    eprintln!("Start database: {:?}", path);
    create_dir_all(&path.parent().expect(format!("Could not get parent of {:?}", path).as_str())).unwrap();
    File::create(&path).expect(format!("Could not create database: {:?}", &path).as_str());
    let conn = Connection::open(&path).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS songs (name TEXT NOT NULL ,album TEXT ,lyrics TEXT)",()).unwrap();
    conn.close().unwrap();
}
fn get_database() -> Connection {
    let path = config().map(|path| path.join("snowstormosd")).unwrap_or_default().join("snowstormosd.sqlite");
    let conn = Connection::open(&path).unwrap();
    return conn;
}

#[cfg(test)]
mod tests {
    use crate::get_metadata;

    #[test]
    fn test_get_metadat() {eprintln!("{:?}", get_metadata());
    assert!(true);}}
