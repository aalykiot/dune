// ANSI Colors Utility
//
// ANSI color codes for safety signs are described in ANSI Z535.1-2011.
// These colors are red, orange, yellow, green, blue, purple, gray, black, white,
// and combinations of black, white, and / or yellow.
//
// https://stackoverflow.com/questions/4842424/list-of-ansi-color-escape-sequences

const ESC = '\u001b';
const CLEAR = ESC + '[0m';
const BOLD = ESC + '[1m';
const UNDERLINE = ESC + '[4m';
const BLACK = ESC + '[30m';
const RED = ESC + '[31m';
const GREEN = ESC + '[32m';
const YELLOW = ESC + '[33m';
const BLUE = ESC + '[34m';
const MAGENTA = ESC + '[35m';
const CYAN = ESC + '[36m';
const WHITE = ESC + '[37m';
const BRIGHT_BLACK = ESC + '[90m';
const BRIGHT_RED = ESC + '[91m';
const BRIGHT_GREEN = ESC + '[92m';
const BRIGHT_YELLOW = ESC + '[93m';
const BRIGHT_BLUE = ESC + '[94m';
const BRIGHT_MAGENTA = ESC + '[95m';
const BRIGHT_CYAN = ESC + '[96m';
const BRIGHT_WHITE = ESC + '[97m';
const BG_BLACK = ESC + '[40m';
const BG_RED = ESC + '[41m';
const BG_GREEN = ESC + '[42m';
const BG_YELLOW = ESC + '[43m';
const BG_BLUE = ESC + '[44m';
const BG_MAGENTA = ESC + '[45m';
const BG_CYAN = ESC + '[46m';
const BG_WHITE = ESC + '[47m';
const BG_BRIGHT_BLACK = ESC + '[100m';
const BG_BRIGHT_RED = ESC + '[101m';
const BG_BRIGHT_GREEN = ESC + '[102m';
const BG_BRIGHT_YELLOW = ESC + '[103m';
const BG_BRIGHT_BLUE = ESC + '[104m';
const BG_BRIGHT_MAGENTA = ESC + '[105m';
const BG_BRIGHT_CYAN = ESC + '[106m';
const BG_BRIGHT_WHITE = ESC + '[107m';

/**
 * Text styling utilities.
 */

export const bold = (value) => BOLD + value + CLEAR;
export const underline = (value) => UNDERLINE + value + CLEAR;

/**
 * ANSI supported colors.
 */

export const black = (value) => BLACK + value + CLEAR;
export const red = (value) => RED + value + CLEAR;
export const green = (value) => GREEN + value + CLEAR;
export const yellow = (value) => YELLOW + value + CLEAR;
export const blue = (value) => BLUE + value + CLEAR;
export const magenta = (value) => MAGENTA + value + CLEAR;
export const cyan = (value) => CYAN + value + CLEAR;
export const white = (value) => WHITE + value + CLEAR;
export const bright_black = (value) => BRIGHT_BLACK + value + CLEAR;
export const bright_red = (value) => BRIGHT_RED + value + CLEAR;
export const bright_green = (value) => BRIGHT_GREEN + value + CLEAR;
export const bright_yellow = (value) => BRIGHT_YELLOW + value + CLEAR;
export const bright_blue = (value) => BRIGHT_BLUE + value + CLEAR;
export const bright_magenta = (value) => BRIGHT_MAGENTA + value + CLEAR;
export const bright_cyan = (value) => BRIGHT_CYAN + value + CLEAR;
export const bright_white = (value) => BRIGHT_WHITE + value + CLEAR;

/**
 * ANSI supported colors for the background.
 */

export const bg_black = (value) => BG_BLACK + value + CLEAR;
export const bg_red = (value) => BG_RED + value + CLEAR;
export const bg_green = (value) => BG_GREEN + value + CLEAR;
export const bg_yellow = (value) => BG_YELLOW + value + CLEAR;
export const bg_blue = (value) => BG_BLUE + value + CLEAR;
export const bg_magenta = (value) => BG_MAGENTA + value + CLEAR;
export const bg_cyan = (value) => BG_CYAN + value + CLEAR;
export const bg_white = (value) => BG_WHITE + value + CLEAR;
export const bg_bright_black = (value) => BG_BRIGHT_BLACK + value + CLEAR;
export const bg_bright_red = (value) => BG_BRIGHT_RED + value + CLEAR;
export const bg_bright_green = (value) => BG_BRIGHT_GREEN + value + CLEAR;
export const bg_bright_yellow = (value) => BG_BRIGHT_YELLOW + value + CLEAR;
export const bg_bright_blue = (value) => BG_BRIGHT_BLUE + value + CLEAR;
export const bg_bright_magenta = (value) => BG_BRIGHT_MAGENTA + value + CLEAR;
export const bg_bright_cyan = (value) => BG_BRIGHT_CYAN + value + CLEAR;
export const bg_bright_white = (value) => BG_BRIGHT_WHITE + value + CLEAR;

export default {
  bold,
  underline,
  black,
  red,
  green,
  yellow,
  blue,
  magenta,
  cyan,
  white,
  bright_black,
  bright_red,
  bright_green,
  bright_yellow,
  bright_blue,
  bright_magenta,
  bright_cyan,
  bright_white,
  bg_black,
  bg_red,
  bg_green,
  bg_yellow,
  bg_blue,
  bg_magenta,
  bg_cyan,
  bg_white,
  bg_bright_black,
  bg_bright_red,
  bg_bright_green,
  bg_bright_yellow,
  bg_bright_blue,
  bg_bright_magenta,
  bg_bright_cyan,
  bg_bright_white,
};
