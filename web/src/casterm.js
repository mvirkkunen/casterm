import "./casterm.css";
import "../node_modules/xterm/dist/xterm.css";
import ReconnectingWebSocket from "reconnecting-websocket";
import { Terminal } from "xterm";
import fit from "xterm/lib/addons/fit/fit";
import attach from "xterm/lib/addons/attach/attach";
import NoSleep from "nosleep.js";

Terminal.applyAddon(fit);
Terminal.applyAddon(attach);

function run() {
    function sendSize() {
        const msg = new Uint8Array(5);

        msg[0] = 1;
        msg[1] = (term.rows >> 8) & 0xff;
        msg[2] = term.rows & 0xff;
        msg[3] = (term.cols >> 8) & 0xff;
        msg[4] = term.cols & 0xff;

        sock.send(msg);
    }

    const sock = new ReconnectingWebSocket("ws://" + location.host + "/ws");
    sock.binaryType = "arraybuffer";

    const term = new Terminal({
        disableStdin: true,
    });

    const noSleep = new NoSleep();

    term.open(document.getElementById("terminal"));
    term.fit();

    term.write("Connecting...");
    //term.write("ws://" + location.host + "/ws");

    term.attach(sock, false, true);

    term.on("resize", e => {
        if (sock.readyState == ReconnectingWebSocket.OPEN) {
            sendSize();
        }
    });

    sock.onopen = () => {
        sendSize();
    };

    sock.onerror = (e) => {
        term.write("websocket error: " + e.message);
    };

    sock.onmessage = (e) => {
        //term.writeUtf8(new Uint8Array(e.data));
    };
    
    window.addEventListener("resize", () => term.fit());

    document.getElementById("fullscreen").addEventListener("click", e => {
        e.preventDefault();

        noSleep.enable();

        term.element.requestFullscreen();
    });

    window.addEventListener("fullscreenchange", e => {
        if (!document.fullscreenElement)
            noSleep.disable();
    })
}

window.addEventListener("DOMContentLoaded", run);