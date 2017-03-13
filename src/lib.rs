#[macro_use] extern crate log;
extern crate svgparser;

use std::str;
use std::mem;

use svgparser::{svg, path, Stream};
use svgparser::path::SegmentData::{self, MoveTo, LineTo, HorizontalLineTo, VerticalLineTo};

pub type CoordinatePair = (f64, f64);
pub type Polyline = Vec<CoordinatePair>;


struct CurrentLine {
    line: Polyline,
}

impl CurrentLine {
    fn new() -> Self {
        CurrentLine { line: Polyline::new() }
    }

    /// Add a CoordinatePair to the internal polyline.
    fn add(&mut self, pair: CoordinatePair) {
        self.line.push(pair);
    }

    /// A polyline is only valid if it has more than 1 CoordinatePair.
    fn is_valid(&self) -> bool {
        self.line.len() > 1
    }

    /// Replace the internal polyline with a new instance and return the
    /// previously stored polyline.
    fn finish(&mut self) -> Polyline {
        let mut tmp = Polyline::new();
        mem::swap(&mut self.line, &mut tmp);
        tmp
    }
}

fn parse_segment_data(data: &SegmentData,
                      current_line: &mut CurrentLine,
                      lines: &mut Vec<Polyline>) {
    match data {
        &MoveTo { x: x, y: y } => {
            if current_line.is_valid() {
                lines.push(current_line.finish());
            }
            current_line.add((x, y));
        },
        &LineTo { x: x, y: y } => {
            current_line.add((x, y));
        },
        d @ _ => {
            println!("Unsupported segment data: {:?}", d);
        }
    }
}

fn parse_path(data: Stream) -> Vec<Polyline> {
    debug!("New path");

    let mut lines = Vec::new();

    let mut p = path::Tokenizer::new(data);
    let mut line = CurrentLine::new();
    loop {
        match p.parse_next() {
            Ok(segment_token) => {
                match segment_token {
                    path::SegmentToken::Segment(segment) => {
                        debug!("  Segment data: {:?}", segment.data);
                        parse_segment_data(&segment.data, &mut line, &mut lines);
                    },
                    path::SegmentToken::EndOfStream => break,
                }
            },
            Err(e) => {
                warn!("Invalid path segment: {:?}", e);
                break;
            },
        }
    }

    lines
}

pub fn parse(svg: &str) -> Result<Vec<Polyline>, String> {
    let bytes = svg.as_bytes();

    let mut polylines = Vec::new();
    let mut tokenizer = svg::Tokenizer::new(&bytes);
    loop {
        match tokenizer.parse_next() {
            Ok(t) => {
                match t {
                    svg::Token::Attribute(name, value) => {
                        // Process only 'd' attributes
                        if name == b"d" {
                            polylines.extend(parse_path(value));
                        }
                    },
                    svg::Token::EndOfStream => break,
                    _ => {},
                }
            },
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(e.to_string());
            }
        }
    }

    Ok(polylines)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
