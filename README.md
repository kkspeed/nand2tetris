# Nand2Tetris
Tweaks to [nand2tetris](http://nand2tetris.org) course projects.

## Decompiler
This project aims to make a decompiler of Nand2Tetris VM code, which (in the
envision), should generate Jack source code from VM code.
[Decompiler](https://raw.githubusercontent.com/kkspeed/nand2tetris/master/image/nand2tetris_decompiler.png)

The decompiler is WIP. Currently it decompiles into untyped IR, which reconstructs
the control flow.

TODO:
- Eliminate <tt>POINTER</tt> variables.
- Add type inference to reconstruct Jack source code. There are cases where the
  type is impossible to infer so we probably need to annotate type info.
