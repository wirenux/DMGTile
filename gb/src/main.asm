SECTION "Vblank Interruprt", ROM0[$0040]
  reti

SECTION "Header", ROM0[$0100]
  jp Start
  ds $0150 - @, 0 ; space for rgbfix header

SECTION "Main Logic", ROM0[$0150]
Start:
  di
  ld sp, $DFFF

.wait_vblank
  ld a, [$FF44]
  cp 144
  jr c, .wait_vblank
  
  xor a
  ld [$FF40], a

  ld hl, SpriteTileData
  ld de, $8000
  ld bc, 16

.copy_tile
  ld a, [hli]
  ld [de], a
  inc de
  dec bc
  ld a, b
  or c
  jr nz, .copy_tile

  ld hl, $FE00
  ld a, 80
  ld [hli], a
  ld a, 84
  ld [hli], a
  ld a, $00
  ld [hli], a
  ld a, $00
  ld [hli], a

  ld a, %10000010
  ld [$FF40], a

.infinite_loop
  halt
  jr .infinite_loop

SECTION "Graphics", ROM0
SpriteTileData:
  INCBIN "tile.bin"
