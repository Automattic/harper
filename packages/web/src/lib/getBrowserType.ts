import { detect } from "detect-browser"

/** Gets the type of browser the user is using. Defaults to Chrome if undecidable. */
export default function getBrowserType(): "chrome" | "firefox" | "edge"{
  let browser = detect(); 

  if (browser == null){
    return "chrome";
  }

  switch (browser.name){
    case "edge":
    case "chrome":
    case "firefox":
      return browser.name;
    case "edge-chromium":
      return "edge";
    default:
      return "chrome";
  }
}
