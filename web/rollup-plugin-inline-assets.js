import fs from "fs";

// Seriously, is there a plugin for this in npm? If there is, I couldn't find it...
export default function inlineAssets(options) {
    const fileName = options.file || "index.html";

    return {
        name: "inline-assets",
    
        generateBundle(opts, bundle) {
            let code = fs.readFileSync(options.html, "utf8");

            for (let key of Object.keys(bundle)) {
                let b = bundle[key];

                if (b.fileName.match(/\.js$/i)) {
                    code = code.replace(/(?=<\/head>)/, `<script>${b.code}</script>\r\n`)

                    delete bundle[key];
                } else if (b.fileName.match(/\.css$/i)) {
                    code = code.replace(/(?=<\/head>)/, `<style type=\"text/css\">${b.source}</style>\r\n`)

                    delete bundle[key];
                }
            }

            bundle["index.html"] = {
                fileName,
                code,
            };
        },
    };
}
