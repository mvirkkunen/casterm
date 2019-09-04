import resolve from "rollup-plugin-node-resolve";
import babel from "rollup-plugin-babel";
import commonjs from "rollup-plugin-commonjs";
import postcss from "rollup-plugin-postcss";
import { terser } from "rollup-plugin-terser";
import inlineAssets from "./rollup-plugin-inline-assets";

export default {
    input: "src/casterm.js",
    output: {
        dir: "../target/",
        format: "iife",
    },
    plugins: [
        resolve(),
        commonjs(),
        babel({
            exclude: "node_modules/**",
        }),
        postcss({
            extensions: [".css"],
            minimize: true,
            extract: true,
        }),
        terser(),
        inlineAssets({
            html: "src/casterm.html",
        })
    ]
};