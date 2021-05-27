const wasm = import('../pkg')
  .catch(console.error);

const R = {
  line: ( ctx, x1, y1, x2, y2 ) => {
    ctx.beginPath();
    ctx.moveTo(x1,y1);
    ctx.lineTo(x2,y2);
    ctx.closePath();
    ctx.stroke();
  },
  circle: ( ctx, x, y, r, fill ) => {
    ctx.beginPath();
    ctx.arc( x, y, r, 0, Math.PI * 2, true );
    ctx.closePath();

    if( fill )
      ctx.fill();
    else
      ctx.stroke();
  },
  string: ( ctx, x, y, str ) => {
    ctx.fillText( str, x, y );
  },
};

const is_valid_url = url => {
  let ret = false;

  ret |= url.startsWith('https://puzz.link/p');
  ret |= url.startsWith('http://pzv.jp/p.html');

  return ret;
};

const sleep = msec => new Promise(resolve => setTimeout(resolve, msec));

Promise.all([wasm]).then(async function([{ parse_url_dblchoco, solve_dblchoco, parse_url_numlin, solve_numlin }]) {
  document.getElementById('button').innerText = 'Solve it!!!';

  button.onclick = async () => { 
    const input = document.getElementById('url-box');
    
    const url = input.value;

    const col_main = 'rgb(40,40,40)';
    const col_sol = 'rgb(0,160,0)';
    const col_alert = 'rgb(160,0,0)';

    let infoDom = document.getElementById('info');

    let button = document.getElementById('button');

    button.innerText = 'Now Solving...';

    await sleep(500);

    if( url.startsWith('https://puzz.link/p') && url.indexOf('dbchoco') != -1 ) {
      const { color, clue, width, height } = JSON.parse(parse_url_dblchoco(url));

      const depth = document.getElementById('depth-input').value;

      const start = new Date();
      const { sol, decided_flag } = JSON.parse(solve_dblchoco(url, depth));
      const end = new Date();
      const elapsedSec = (end-start) / 1000;

      button.innerText = 'Solve it!!!';

      infoDom.innerText = `実行時間: ${elapsedSec.toFixed(2)} s`;

      console.log({ color, clue, width, height });

      console.log(sol);

      const col_black = 'rgb(204,204,204)';
      const col_white = 'rgb(255,255,255)';

      const canvas = document.getElementById('canvas');
      const ctx = canvas.getContext('2d');
    
      const pad = 20;
      const scrW = 640, scrH = 480;
    
      const s = Math.min( (scrW-pad*2)/width, (scrH-pad*2)/height );
    
      ctx.clearRect(0, 0, scrW, scrH);
    
      for( let i = 0; i < height; ++i ) for( let j = 0; j < width; ++j ) {
        const x = pad + s*j;
        const y = pad + s*i;

        ctx.fillStyle = color[i*width+j] ? col_white : col_black;
        ctx.fillRect(x, y, s, s);
    
        if( clue[i*width+j] != 0 ) {
          const rat = s/40;
          ctx.font = `normal ${Math.floor(30*rat)}px 'Yu Gothic'`;
          ctx.fillStyle = col_main;

          if( clue[i*width+j] < 10 )
            R.string(ctx, x+s/2-8*rat, y+s/2+12*rat, clue[i*width+j] );
          else
            R.string(ctx, x+s/2-17*rat, y+s/2+12*rat, clue[i*width+j] );
        }
      }

      for( let i = 0; i <= height; ++i ) {
        ctx.setLineDash(!i || i == height ? [] : [12]);
        ctx.strokeStyle = col_main;
        ctx.lineWidth = !i || i == height ? 3 : 1;  

        const y = pad + s*i;

        R.line(ctx, pad, y, pad + s*width, y);

        ctx.setLineDash([]);
      }

      for( let j = 0; j <= width; ++j ) {
        ctx.setLineDash(!j || j == width ? [] : [12]);
        ctx.strokeStyle = col_main;  
        ctx.lineWidth = !j || j == width ? 3 : 1;  

        const x = pad + s*j;

        R.line(ctx, x, pad, x, pad + s*height);

        ctx.setLineDash([]);
      }

      let row = 1, col = 0;

      for( let i = 0; i < sol.length; ++i ) {
        if( sol[i] != 'x' ) {
          if( sol[i] == '-' ) {
            ctx.strokeStyle = col_sol;
            ctx.lineWidth = 4;
          } else {
            ctx.strokeStyle = col_alert;
            ctx.lineWidth = 4;
            ctx.setLineDash([12]);
          }

          if( row % 2 == 0 ) {
            const y = pad + s*row/2;
            const x = pad + s*col;

            R.line(ctx, x, y, x+s, y);
          } else {
            const y = pad + s*(row-1)/2;
            const x = pad + s*(col+1);

            R.line(ctx, x, y, x, y+s);
          }

          if( sol[i] == ' ' ) {
            ctx.setLineDash([]);
          }
        }

        ++col;

        if( row % 2 == 0 && col >= width ) {
          col = 0;
          ++row;
        } else if( row % 2 == 1 && col >= width-1 ) {
          col = 0;
          ++row;
        }
      }

      if( !decided_flag ) {
        infoDom.innerText += ' (未確定の境界があります)';
      }
    } else if( is_valid_url(url) && url.indexOf('numlin') != -1 ) {
      const { field, width, height } = JSON.parse(parse_url_numlin(url));
      const { sol } = JSON.parse(solve_numlin(url));
    
      console.log(field);
    
      console.log(sol);
    
      if( field === '' || sol === '' ) {
        console.error('Solver Failed!!!');
    
        return;
      }
    
      const canvas = document.getElementById('canvas');
      const ctx = canvas.getContext('2d');
    
      const pad = 20;
      const scrW = 640, scrH = 480;
    
      const s = Math.min( (scrW-pad*2)/width, (scrH-pad*2)/height );
    
      ctx.clearRect(0, 0, scrW, scrH);
    
      for( let i = 0; i < height; ++i ) for( let j = 0; j < width; ++j ) {
        const x = pad + s*j;
        const y = pad + s*i;
    
        ctx.strokeStyle = 'rgb(40,40,40)';
        ctx.lineWidth = 2;
    
        if( field[i*width+j] != 0 ) {
          const rat = s/40;
          ctx.font = `normal ${Math.floor(30*rat)}px 'Yu Gothic'`;

          if( field[i*width+j] < 10 )
            R.string(ctx, x+s/2-8*rat, y+s/2+12*rat, field[i*width+j] );
          else
            R.string(ctx, x+s/2-17*rat, y+s/2+12*rat, field[i*width+j] );
        }
      }

      for( let i = 0; i <= height; ++i ) {
        ctx.setLineDash(!i || i == height ? [] : [12]);
        ctx.strokeStyle = col_main;
        ctx.lineWidth = !i || i == height ? 3 : 1;  

        const y = pad + s*i;

        R.line(ctx, pad, y, pad + s*width, y);

        ctx.setLineDash([]);
      }

      for( let j = 0; j <= width; ++j ) {
        ctx.setLineDash(!j || j == width ? [] : [12]);
        ctx.strokeStyle = col_main;  
        ctx.lineWidth = !j || j == width ? 3 : 1;  

        const x = pad + s*j;
        
        R.line(ctx, x, pad, x, pad + s*height);

        ctx.setLineDash([]);
      }
    
      const calc = i => pad + s*i + s/2;
    
      for( const arc of sol ) {
        ctx.strokeStyle = col_sol;
        ctx.lineWidth = 4;

        const ver = arc[1][0]-arc[0][0];
        const hor = arc[1][1]-arc[0][1];
        const mar1 = field[arc[0][0]*width+arc[0][1]] > 0 ? s/8*3 : 0;
        const mar2 = field[arc[1][0]*width+arc[1][1]] > 0 ? s/8*3 : 0;

        R.line( ctx, calc(arc[0][1]) + hor*mar1, calc(arc[0][0]) + ver*mar1, calc(arc[1][1]) - hor*mar2, calc(arc[1][0]) - ver*mar2 );
      }
    } else {
      button.innerText = 'Solve it!!!';
      infoDom.innerText = '対応していない URL です';
    }
  }
});