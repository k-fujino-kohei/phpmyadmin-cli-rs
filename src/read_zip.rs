use anyhow::bail;
use async_compression::tokio::bufread::{DeflateDecoder, GzipDecoder};
use bytes::Bytes;
use tokio::io::{AsyncBufRead, AsyncReadExt};

#[derive(Debug)]
pub struct LocalFile {
    pub header: LocalHeader,
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub struct LocalHeader {
    pub file_name: String,
    pub compression_method: u16,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}

pub async fn read_zip(zip: Bytes) -> anyhow::Result<Vec<LocalFile>> {
    let mut files = vec![];
    let mut reader = GzipDecoder::new(zip.as_ref());
    while let Some(file) = read_local_file(&mut reader).await? {
        files.push(file);
    }
    Ok(files)
}

const LOCAL_FILE_HEADER_SIGNATURE: u32 = 0x04034b50;

async fn read_local_file<R>(reader: &mut GzipDecoder<R>) -> anyhow::Result<Option<LocalFile>>
where
    R: AsyncBufRead + std::marker::Unpin,
{
    let header = if let Some(h) = read_local_header(reader).await? {
        h
    } else {
        return Ok(None);
    };

    match header.compression_method {
        8 => {
            let mut file_content = vec![0; header.compressed_size as usize];
            reader.read_exact(&mut file_content).await?;
            let mut reader = DeflateDecoder::new(&*file_content);
            let mut content = Vec::with_capacity(header.uncompressed_size as usize);
            reader.read_to_end(&mut content).await?;
            Ok(Some(LocalFile { header, content }))
        }
        _ => bail!("unsupported compression method."),
    }
}

async fn read_local_header<R>(reader: &mut GzipDecoder<R>) -> anyhow::Result<Option<LocalHeader>>
where
    R: AsyncBufRead + std::marker::Unpin,
{
    let signature = reader.read_u32_le().await?;
    if signature != LOCAL_FILE_HEADER_SIGNATURE {
        return Ok(None);
    }
    let _version_made_by = reader.read_u16_le().await?;
    let _flags = reader.read_u16_le().await?;
    let compression_method = reader.read_u16_le().await?;
    let _last_mod_time = reader.read_u16_le().await?;
    let _last_mod_date = reader.read_u16_le().await?;
    // TODO: Check crc32
    let _crc32 = reader.read_u32_le().await?;
    let compressed_size = reader.read_u32_le().await?;
    let uncompressed_size = reader.read_u32_le().await?;
    let file_name_length = reader.read_u16_le().await? as usize;
    let extra_field_length = reader.read_u16_le().await? as usize;
    let mut file_name_raw = vec![0; file_name_length];
    reader.read_exact(&mut file_name_raw).await?;
    let mut extra_field = vec![0; extra_field_length];
    reader.read_exact(&mut extra_field).await?;
    let file_name = String::from_utf8_lossy(&file_name_raw).into_owned();
    Ok(Some(LocalHeader {
        file_name,
        compression_method,
        compressed_size,
        uncompressed_size,
    }))
}
