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
  
  でかい:
  https://puzz.link/p?dbchoco/16/16/vf9cnls83q419uuv0pgaocl06um0fb1i0pgfnnuo2055f6ulr9bgh41g2j2k3r3g3k2g22g2q5h5i1l5l5l2k5j4i2g5n51m5o4p5j4g2g5n51j2j5k4l5l5k1g1h5h5j2l2j2i1n11o32p3g
  
  https://puzz.link/p?dbchoco/12/12/7orhfgfc3i1ugsce2v0jgds3t3m7o2g1i3j2i3h51g1g3o5o5g6h4i3k61h4h32k2h3n4j3g7g6g2h4w5j7h23g3g35g3p3
  https://puzz.link/p?dbchoco/8/8/0c5hu1vlvn4hgm45h6h5p6g5o6p5i5g5i5j2

  こける:
  https://puzz.link/p?dbchoco/6/6/poc4f1tgj6zl3h3g

*/


#[derive(Serialize, Deserialize, Debug)]
struct NumlinField {
  field: Vec<i32>,
  width: usize, 
  height: usize
}

#[derive(Serialize, Deserialize, Debug)]
struct NumlinSol {
  sol: Vec<Vec<Vec<usize>>>
}

#[wasm_bindgen]
pub fn parse_url_numlin(url: String) -> String {
  let opt = parse_url_numlin_internal(url);

  if let Some(clue) = opt {
    let width = clue.width() as usize;
    let height = clue.height() as usize;

    let mut clue_vec = vec![];

    for i in 0..width*height {
      match clue[i] {
        numberlink::Clue(x) => clue_vec.push(x),
        _ => {}
      }
    }

    let payload = NumlinField {
      field: clue_vec,
      width: width, 
      height: height
    };

    return serde_json::to_string(&payload).unwrap();
  } else {

    return "".to_string();
  }
}

#[wasm_bindgen]
pub fn solve_numlin(url: String) -> String {
  let opt = parse_url_numlin_internal(url);

  if let Some(clue) = opt {
    let ans = numberlink::solve2(&clue, None, false, false);
    let lines = ans.answers;

    let mut sol_vec: Vec<Vec<Vec<usize>>> = vec![];

    for line in &lines {
      let width = line.width() as usize;
      let height = line.height() as usize;

      for i in 0..height*(width-1) {
        let row = i/(width-1);
        let col = i%(width-1);

        if line.right(P(row as i32, col as i32)) {
          sol_vec.push(vec![vec![row, col], vec![row, col+1]]);
        }
      }

      for i in 0..width*(height-1) {
        let col = i/(height-1);
        let row = i%(height-1);

        if line.down(P(row as i32, col as i32)) {
          sol_vec.push(vec![vec![row, col], vec![row+1, col]]);
        }
      }
    }

    let payload = NumlinSol {
      sol: sol_vec,
    };

    return serde_json::to_string(&payload).unwrap();
  } else {
    return "".to_string();
  }
}

fn parse_url_numlin_internal(url: String) -> Option<Grid<numberlink::Clue>> {
  let splitter = '/';
  let params: Vec<String> = url.split(splitter).map(|s| s.to_string()).collect();
  let length = params.len();

  if length < 3 {
      return None;
  }

  let width = params[length-3].parse().unwrap_or(0);
  let height = params[length-2].parse().unwrap_or(0);

  if width <= 0 || height <= 0 {
      return None;
  }

  let field_code = params[length-1].clone();

  if !is_valid_code(&field_code) {
      return None;
  }

  decode_field(width, height, field_code)
}

fn is_valid_code(code: &String) -> bool {
  return code.chars().all(|ch| char::is_alphanumeric(ch) || ch == '-');
}

fn decode_field(width: usize, height: usize, code: String) -> Option<Grid<numberlink::Clue>> {
  let list: &Vec<char> = &code.chars().collect();
  let mut index: usize = 0;
  let mut i: usize = 0;
  let mut j: usize = 0;

  let mut res: Vec<Vec<usize>> = vec![vec![0; width as usize]; height as usize];

  while index < list.len() {
      while let Some(num) = get_num(&mut index, list) {
          res[i as usize][j as usize] = num;

          j += 1;

          if j >= width {
              j = 0;
              i += 1;
          }

          if index >= list.len() {
              break;
          }
      }

      consume(&mut index, &mut i, &mut j, width, list);
  }

  let mut clue = Grid::new(height as i32, width as i32, numberlink::Clue(0));

  for i in 0..height {
    for j in 0..width {
      clue[i*width+j] = numberlink::Clue(res[i][j] as i32);
    }
  }

  Some(clue)
}

fn get_num(index: &mut usize, list: &Vec<char>) -> Option<usize> {
  let ch = list[*index as usize];

  if ch == '-' {
      *index += 1;
      let mut res = 0;

      while res*16 < 100 && *index < list.len() && list[*index].is_digit(16) {
          res *= 16;
          res += list[*index].to_digit(16).unwrap();
          *index += 1;
      }

      return Some(res as usize);
  } else if ch.is_digit(16) {
      *index += 1;

      return match ch.to_digit(16) {
          Some(num) => Some(num as usize),
          None => None,
      };
  } else {
      return None;
  }
}

fn consume(index: &mut usize, i: &mut usize, j: &mut usize, width: usize, list: &Vec<char>) {
  let length = list.len();

  while *index < length && !(list[*index as usize]).is_digit(16) {
      let ch = list[*index as usize];
      let value = (ch as i32) - ('f' as i32);

      if value <= 0 {
          return;
      }

      let value: usize = value as usize;

      *j += value;

      if *j >= width {
          *i += *j/width;
          *j %= width;
      }

      *index += 1;
  }    
}

fn main() {
  let url = "https://puzz.link/p?numlin/42/25/zzi1zx5j3ve-1cv-13n6zp-2br2zl-2cvep8-1dp-29z-10x7zj-14t-1dzn-16j-1abj-20zr-19l-21zv-1bh-1fzg2h6hbh-11hch-17l-22h-24h-27h-2ah-24h-1cj3h7hch-12h-16h-1al-14h-21h-23h-29h-26h-2cj4h8hdh9h-17h-12l-1fh-25h-28h-2bh-27h-10zgah-20zv-1bl-23zr-18j-18fj-22zn-15t-26zj-13x-25zfp-15-19p-11vazl4r9zp-2an5vd-1ev-1ej1zx-28zzi".to_string();

  parse_url_numlin(url);
}