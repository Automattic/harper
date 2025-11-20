import Color from "colorjs.io"

/** Get the text color that best contrasts with a background of the provided color. */
export function getConstrastingTextColor(color: string): "black" | "white"{
  const c = new Color(color);
  const luminance = c.luminance;

  if (luminance > 0.5){
return "black";
  }else{
return "white";
  }
}
