mod common;
mod decoder;
mod encoder;
mod symbol_stats;

use decoder::*;
use encoder::*;
use symbol_stats::*;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;

fn encode(buffer: Vec<u8>, stats: &SymbolStats) -> io::Result<Vec<u32>> {
    let mut out_buf: Vec<u32> = Vec::new();
    let mut enc_symbols: Vec<EncoderSymbol> = Vec::new();

    for i in 0..256 {
        enc_symbols.push(EncoderSymbol::new(
            stats.cum_freqs[i],
            stats.freqs[i],
            common::PROB_BITS,
        ));
    }

    let mut encoder = BoxEncoder::new();
    for i in (1..buffer.len() + 1).rev() {
        let s = buffer[i as usize - 1] as usize;
        encoder.put_symbol(&mut out_buf, &enc_symbols[s], common::PROB_BITS);
    }
    encoder.flush(&mut out_buf);
    println!(
        "rANS encode: {} bytes => {} bytes",
        buffer.len(),
        out_buf.len() * 4
    );

    Ok(out_buf)
}

fn decode(mut buffer: Vec<u32>, stats: &SymbolStats, in_size: usize) -> io::Result<Vec<u8>> {
    let encoded_size = buffer.len() * 4;
    let mut out_buf: Vec<u8> = Vec::new();
    let mut dec_symbols: Vec<DecoderSymbol> = Vec::new();

    for i in 0..256 {
        dec_symbols.push(DecoderSymbol::new(stats.cum_freqs[i], stats.freqs[i]));
    }

    let mut cum2sym = [0u8; common::PROB_SCALE as usize];
    for s in 0..256 {
        for i in stats.cum_freqs[s]..stats.cum_freqs[s + 1] {
            cum2sym[i as usize] = s as u8;
        }
    }

    let mut decoder = BoxDecoder::new(&mut buffer);
    for _ in 0..in_size {
        let s = cum2sym[decoder.get(common::PROB_BITS) as usize]; 
        out_buf.push(s);
        decoder.advance_symbol(&mut buffer, &dec_symbols[s as usize], common::PROB_BITS);
    }
    println!(
        "rANS decode: {} bytes => {} bytes",
        encoded_size,
        out_buf.len()
    );

    Ok(out_buf)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);

    if args.len() < 2 {
        println!("Specify files using {} [files]", args[0]);
        return Ok(());
    }

    let mut in_sizes = Vec::new();
    let mut buffers = Vec::new();

    for file in &args[1..] {
        let mut buffer = Vec::new();
        let mut f = File::open(file)?;
        in_sizes.push(f.metadata()?.len());
        f.read_to_end(&mut buffer)?;
        buffers.push(buffer);
    }

    let mut single_buffer = Vec::new();
    for b in &buffers {
        single_buffer.append(&mut b.clone());
    }

    let mut single_in_size = 0; 
    for s in &in_sizes {
        single_in_size += s;
    }

    /*let mut f = File::open("book1.txt")?;
    let in_size = f.metadata().unwrap().len();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;*/

    let mut stats = SymbolStats::new();
    stats.count_freqs(&single_buffer);
    stats.normalise_freqs(common::PROB_SCALE);

    println!("Single buffer");
    let encoded = encode(single_buffer, &stats)?;
    println!("Size: {}", encoded.len() * 4);
    println!("Ratio: {}", ((encoded.len() * 4) as f32 / single_in_size as f32 ) * 8.0);
    let decoded = decode(encoded, &stats, single_in_size as usize)?;

    println!("Multiple buffers");
    let mut total_size = 0;
    for i in 0..buffers.len() {
        let mut current_buffer = buffers.pop().unwrap();
        let mut current_size = in_sizes.pop().unwrap();
        let mut stats = SymbolStats::new();
        stats.count_freqs(&current_buffer);
        stats.normalise_freqs(common::PROB_SCALE);

        let encoded = encode(current_buffer, &stats)?;
        total_size += encoded.len() * 4;
        let decoded = decode(encoded, &stats, current_size as usize)?;
    }
    println!("Size: {}", total_size);
    println!("Ratio: {}", (total_size as f32 / single_in_size as f32) * 8.0);

    /*let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open("book1.txt.box")?;

    for &i in encoded.iter() {
        f.write_u32::<LittleEndian>(i)?;
    }*/
    Ok(())
}
