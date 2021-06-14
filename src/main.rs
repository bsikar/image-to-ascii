#[macro_use]
extern crate clap;

use clap::Arg;
use image::imageops::{resize, FilterType};
use image::{GenericImageView, GrayImage, ImageBuffer, Pixel};
use std::fmt::{self, Display, Formatter};
use std::io::prelude::Write;
use FilterType::Lanczos3;

const ASCII_TABLE: [char; 70] = [
    '$', '@', 'B', '%', '8', '&', 'W', 'M', '#', '*', 'o', 'a', 'h', 'k', 'b', 'd', 'p', 'q', 'w',
    'm', 'Z', 'O', '0', 'Q', 'L', 'C', 'J', 'U', 'Y', 'X', 'z', 'c', 'v', 'u', 'n', 'x', 'r', 'j',
    'f', 't', '/', '\\', '|', '(', ')', '1', '{', '}', '[', ']', '?', '-', '_', '+', '~', '<', '>',
    'i', '!', 'l', 'I', ';', ':', ',', '"', '^', '`', '\'', '.', ' ',
];

struct Scaler(Option<u32>, Option<u32>);

impl Scaler {
    fn scale<I: GenericImageView>(
        &self,
        image: &I,
        filter: FilterType,
    ) -> ImageBuffer<I::Pixel, Vec<<I::Pixel as Pixel>::Subpixel>>
    where
        I::Pixel: 'static,
        <I::Pixel as Pixel>::Subpixel: 'static,
    {
        match (self.0, self.1) {
            (Some(width), Some(height)) => resize(image, width, height, filter),

            (Some(width), None) => {
                let height = width as f64 / image.width() as f64 * image.height() as f64;

                resize(image, width, height as u32, filter)
            }

            (None, Some(height)) => {
                let width = height as f64 / image.height() as f64 * image.width() as f64;
                resize(image, width as u32, height, filter)
            }

            // TODO maybe fix lmao | make suck less
            (None, None) => ImageBuffer::from_raw(
                image.width(),
                image.height(),
                image
                    .pixels()
                    .map(|(_, _, pixel)| {
                        pixel
                            .channels()
                            .iter()
                            .map(|p| *p)
                            .collect::<Vec<<I::Pixel as Pixel>::Subpixel>>()
                    })
                    .flatten()
                    .collect(),
            )
            .expect("failed to build image buffer"),
        }
    }
}

struct AsciiImage(GrayImage);

impl Display for AsciiImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let text = self
            .0
            .rows()
            .map(|row| {
                row.map(|luma| {
                    // luma.0 == luma.channels() *NOTE* it is used here because formating
                    ASCII_TABLE[(luma.0[0] as f64 / u8::MAX as f64 * (ASCII_TABLE.len() - 1) as f64)
                        .trunc() as usize]
                        .to_string()
                        .repeat(2)
                })
                .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}", text)
    }
}

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("input")
                .help("This is the image you are converting to ascii")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .help("This is the output name")
                .index(2),
        )
        .arg(
            Arg::with_name("width")
                .help("This is your output image width in charaters")
                .short("w")
                .long("width")
                .value_name("width"),
        )
        .arg(
            Arg::with_name("height")
                .help("This is your output image height in charaters")
                .short("h")
                .long("height")
                .value_name("height"),
        )
        .get_matches();

    let width = matches
        .value_of("width")
        .map(|x| x.parse::<u32>().expect("failed to parse to u32"));
    let height = matches
        .value_of("height")
        .map(|x| x.parse::<u32>().expect("failed to parse to u32"));

    let scaler = Scaler(width, height);

    let img = image::open(matches.value_of("input").unwrap())
        .expect("Failed to open image.")
        .into_luma8();
    let img = scaler.scale(&img, Lanczos3);

    if let None = matches.value_of("output") {
        println!("{}", AsciiImage(img));
    } else {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(matches.value_of("output").unwrap())
            .expect("failed to create file");
        file.write_all(format!("{}", AsciiImage(img)).as_bytes())
            .expect("failed to write to file");
    }
}
