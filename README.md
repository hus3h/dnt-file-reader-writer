# dnt-file-reader-writer

Small library to read and write `.dnt` (Dragon Nest Table) files. Check out [dntviewer](https://github.com/spacem/dntviewer) for a graphical viewer.

**Note: this was made for learning purposes only and was tested on an old `.dnt` file that was used for a private server a while ago.**

___

## Usage

```rust
use dnt_file_reader_writer::{DntFileReader, DntFileWriter};
use std::fs::File;

fn main() {
    let mut reader = DntFileReader::new(
        File::open("/path/to/file.dnt").unwrap(),
    );

    let mut writer = DntFileWriter::new(
        File::create("/path/to/new-file.dnt").unwrap(),
    );

    reader.read().unwrap();

    writer.write(reader.data()).unwrap();
}
```
