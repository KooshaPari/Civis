import { Config } from "@remotion/cli/config";

Config.setVideoImageFormat("jpeg");
Config.setOverwriteOutput(true);
// public/ is staged per-render by bin/render.mjs.
Config.setPublicDir("public");
