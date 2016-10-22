use uuid::Uuid;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::remove_file;
use std::io::Read;
use std::io::Write;

use bincode::rustc_serialize::{encode, decode, encode_into, decode_from};
use bincode::SizeLimit;
use rustc_serialize::json;
use bincode::rustc_serialize::{EncodingError, DecodingError};

use std::io::Error as IoError;
use std::io::{Seek, SeekFrom};

use document::Document;

pub struct ioHandler;

impl ioHandler {
    /// Decodes document from file by given id
    /// # Arguments
    ///
    /// * `id` - The uuid of the document in order to find the document
    /// * `volume` - The folder where the documents are saved
    pub fn get(id: Uuid, volume: &str) -> Result<Document, DecodingError> {
        let mut handler = try!(File::open(format!("{}/{}", volume, id)));

        let mut decoded: Document = try!(decode_from(&mut handler, SizeLimit::Infinite));

        Ok(decoded)
    }

    /// Encodes a document and writes it into a file
    /// # Arguments
    ///
    /// * `document` - The document which will be encoded
    /// * `volume` - The folder where the documents are saved
    pub fn post(document: Document, volume: &str) -> Result<Uuid, EncodingError> {
        // TODO implement error-handling
        let mut handler = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .open(format!("{}/{}", volume, &document.id))
            .unwrap();

        try!(encode_into(&document, &mut handler, SizeLimit::Infinite));

        Ok(document.id)
    }

    /// Deletes a document
    /// # Arguments
    ///
    /// * `id` - The uuid of the document in order to find the document
    /// * `volume` - The folder where the documents are saved
    pub fn remove(id: Uuid, volume: &str) -> Result<String, IoError> {
        try!(remove_file(format!("{}/{}", volume, id)));
        Ok("Document deleted".to_string())
    }

    pub fn put(id: Uuid, payload: &[u8], volume: &str) -> Result<String, IoError> {
        let mut handler = try!(OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(format!("{}/{}", volume, id)));

        let mut document: Document = decode_from(&mut handler, SizeLimit::Infinite)
            .expect(&format!("Cannot find file {}/{}", volume, id));

        handler.seek(SeekFrom::Start(0));

        document.put(payload.to_vec());

        encode_into(&document, &mut handler, SizeLimit::Infinite);

        Ok("Documented updated".to_string())
    }
}

#[cfg(test)]
mod tests {
    use document::Document;
    use uuid::Uuid;
    use std::fs::{create_dir, remove_dir, File, metadata};
    use io_handler::ioHandler;

    static VOLUME: &str = "test_dir";

    struct Setup;

    impl Setup {
        pub fn init_dir(&self) {
            create_dir(VOLUME);
        }
    }

    #[test]
    fn test_post() {
        let setup = Setup;
        setup.init_dir();

        let doc = Document {
            id: Uuid::new_v4(),
            payload: Vec::new(),
            version: 0,
        };

        ioHandler::post(doc, VOLUME).unwrap();
    }

    #[test]
    fn test_get() {
        let setup = Setup;

        let id = Uuid::new_v4();
        let bytes = b"Hello world".to_vec();

        let doc = Document {
            id: id,
            payload: bytes,
            version: 0,
        };

        ioHandler::post(doc.clone(), VOLUME).unwrap();

        let doc2 = ioHandler::get(id, VOLUME).unwrap();

        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_put() {
        let setup = Setup;

        let id = Uuid::new_v4();
        let bytes = b"Hello world".to_vec();

        let doc = Document {
            id: id,
            payload: bytes,
            version: 0,
        };

        ioHandler::post(doc.clone(), VOLUME).unwrap();

        let updated_payload = b"This is updated! :P";

        ioHandler::put(id.clone(), updated_payload, VOLUME).unwrap();

        let doc2 = ioHandler::get(id, VOLUME).unwrap();

        assert_eq!(doc2.payload, b"This is updated! :P");
    }

    #[test]
    fn test_remove() {
        let setup = Setup;

        let id = Uuid::new_v4();
        let bytes = b"Hello world".to_vec();

        let doc = Document {
            id: id,
            payload: bytes,
            version: 0,
        };

        ioHandler::post(doc.clone(), VOLUME).unwrap();

        ioHandler::remove(id, VOLUME).unwrap();

        match metadata(format!("{}/{}", VOLUME, id)) {
            Ok(ref attr) if attr.is_file() == true => {
                assert!(false);
            } 
            _ => assert!(true),
        }
    }
}
