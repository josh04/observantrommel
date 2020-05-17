import * as wasm from "sechit";

//var promise = wasm.main_nonthread();
document.getElementById("loading").innerHTML = "";
document.getElementById("loading").style.padding = "";
document.body.classList += "change";
document.body.style.backgroundColor = "rgb(110, 122, 111)";

window.wasm = wasm;
sechit = window.wasm;
game = new wasm.GameState;