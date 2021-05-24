use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use serde::{Serialize, Deserialize};

mod common;
mod io;

mod doublechoco;
mod numberlink;
mod grid_loop;
mod slitherlink;

use common::*;
use doublechoco::*;
use std::env;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Serialize, Deserialize, Debug)]
struct DblchocoField {
  color: Vec<bool>,
  clue: Vec<i32>,
  width: usize,
  height: usize
}

fn parse_url_dblchoco_internal(url: &str) -> (Grid<Color>, Grid<Clue>) {
    let tokens = url.split("/").collect::<Vec<_>>();
    let width = tokens[tokens.len() - 3].parse::<i32>().unwrap();
    let height = tokens[tokens.len() - 2].parse::<i32>().unwrap();
    let body = tokens[tokens.len() - 1].chars().collect::<Vec<char>>();

    let mut color = Grid::new(height, width, Color::White);
    let mut clue = Grid::new(height, width, NO_CLUE);

    let mut idx = 0usize;
    for i in 0..((height * width + 4) / 5) {
        let v = body[idx];
        idx += 1;
        let bits = if '0' <= v && v <= '9' {
            (v as i32) - ('0' as i32)
        } else {
            (v as i32) - ('a' as i32) + 10
        };
        for j in 0..5 {
            let p = i * 5 + j;
            let y = p / width;
            let x = p % width;
            if y < height {
                color[P(y, x)] = if (bits & (1 << (4 - j))) != 0 {
                    Color::Black
                } else {
                    Color::White
                };
            }
        }
    }
    fn convert_hex(v: char) -> i32 {
        if '0' <= v && v <= '9' {
            (v as i32) - ('0' as i32)
        } else {
            (v as i32) - ('a' as i32) + 10
        }
    }
    let mut pos = 0;
    while idx < body.len() {
        if 'g' <= body[idx] {
            pos += (body[idx] as i32) - ('f' as i32);
            idx += 1;
        } else {
            let val;
            if body[idx] == '-' {
                val = convert_hex(body[idx + 1]) * 16 + convert_hex(body[idx + 2]);
                idx += 3;
            } else {
                val = convert_hex(body[idx]);
                idx += 1;
            }
            clue[P(pos / width, pos % width)] = val;
            pos += 1;
        }
    }

    (color, clue)
}

#[wasm_bindgen]
pub fn parse_url_dblchoco(url: &str) -> String {
  let (color, clue) = parse_url_dblchoco_internal(url);

  let mut color_vec = vec![];

  let width = color.width() as usize;
  let height = color.height() as usize;

  for i in 0..width*height {
    color_vec.push(color[i]);
  }

  let color_fls = color_vec.clone().into_iter().map(|x| match x {
    Color::Black => false,
    Color::White => true,
  }).collect::<Vec<bool>>();

  let mut clue_vec = vec![];

  for i in 0..width*height {
    clue_vec.push(clue[i]);
  }

  let payload = DblchocoField {
    color: color_fls,
    clue: clue_vec,
    width: width,
    height: height
  };

  serde_json::to_string(&payload).unwrap()
}

#[wasm_bindgen]
pub fn solve_dblchoco(url: &str) -> String {
  let (color, clue) = parse_url_dblchoco_internal(url);

  let height = color.height();
  let width = color.width();

  let mut field = Field::new(&color, &clue);
  field.trial_and_error(2);

  assert_eq!(field.inconsistent(), false);

  let mut ans = "".to_string();

  for y in 0..(height * 2 + 1) {
      for x in 0..(width * 2 + 1) {
          match (y % 2, x % 2) {
              (0, 0) => {},
              (0, 1) => {
                  if !(y == 0 || y == height * 2) {
                      match field.border(LP(y - 1, x - 1)) {
                          Border::Undecided => ans += " ",
                          Border::Line => ans += "-",
                          Border::Blank => ans += "x",
                      }
                  }
              },
              (1, 0) => {
                  if !(x == 0 || x == width * 2)  {
                      match field.border(LP(y - 1, x - 1)) {
                          Border::Undecided => ans += " ",
                          Border::Line => ans += "-",
                          Border::Blank => ans += "x",
                      }
                  }
              },
              (1, 1) => {},
              _ => unreachable!(),
          }
      }
  }

  ans
}

/*
  https://puzz.link/p?dbchoco/6/6/vj801ovgk4r4g4l2h3j2
  でかい
  https://puzz.link/p?dbchoco/16/16/vf9cnls83q419uuv0pgaocl06um0fb1i0pgfnnuo2055f6ulr9bgh41g2j2k3r3g3k2g22g2q5h5i1l5l5l2k5j4i2g5n51m5o4p5j4g2g5n51j2j5k4l5l5k1g1h5h5j2l2j2i1n11o32p3g
  https://puzz.link/p?dbchoco/12/12/7orhfgfc3i1ugsce2v0jgds3t3m7o2g1i3j2i3h51g1g3o5o5g6h4i3k61h4h32k2h3n4j3g7g6g2h4w5j7h23g3g35g3p3
  https://puzz.link/p?dbchoco/8/8/0c5hu1vlvn4hgm45h6h5p6g5o6p5i5g5i5j2
*/