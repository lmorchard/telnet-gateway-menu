#!/usr/bin/env node
require("dotenv").config();

const { HOST = "0.0.0.0", PORT = "2323", LOG_LEVEL = "trace" } = process.env;

const net = require("net");
const { v4: uuidv4 } = require("uuid");
const EventEmitter = require("events");
const {
  TelnetSocket,
  TelnetSpec: { Commands, Options, SubNegotiationCommands },
} = require("telnet-socket");
const pino = require("pino");
const log = pino({ level: LOG_LEVEL });

const addressBook = [
  ["Particles", "particlesbbs.dyndns.org:6400"],
  ["Level29", "bbs.fozztexx.com:23"],
];

async function main() {
  const app = new TelnetGatewayMenu();
  app.listen();
}

class TelnetGatewayMenu extends EventEmitter {
  constructor(options = {}) {
    super();
    this.options = {
      HOST,
      PORT,
      ...options,
    };
    this.id = this.genID();
    this.server = net.createServer();
    this.connections = {};
    this.log = pino({ level: LOG_LEVEL }).child({
      name: "TelnetGatewayMenu",
      id: this.id,
    });
  }

  genID() {
    return uuidv4();
  }

  listen() {
    const { HOST, PORT } = this.options;
    this.server.on("connection", async (socket) =>
      this.acceptConnection(socket)
    );
    this.server.listen(PORT);
    log.info({ msg: "listening", HOST, PORT });
  }

  async acceptConnection(socket) {
    const connection = new TelnetGatewayConnection(this, socket);
    this.connections[connection.id] = connection;
    connection.on("end", () => delete this.connections[connection.id]);
    await connection.runMenu();
    connection.destroy();
  }
}

class TelnetGatewayConnection extends EventEmitter {
  constructor(parent, socket) {
    super();
    this.parent = parent;
    this.id = this.parent.genID();
    this.socket = socket;
    this.log = this.parent.log.child({
      name: "TelnetGatewayConnection",
      id: socket.id,
    });
    this.log.info({ msg: `connected`, ...socket.address() });
    socket.on("end", () => {
      log.info("disconnected");
      this.emit("end");
    });
  }

  destroy() {
    this.socket.destroy();
  }

  async runMenu() {
    const socket = this.socket;
    socket.write(`Hello ${this.id}\r\n`);
    while (!socket.destroyed) {
      try {
        socket.write(`\r\nAddress book:\r\n`);
        for (let idx = 0; idx < addressBook.length; idx++) {
          const [label, address] = addressBook[idx];
          socket.write(
            `${idx.toString().padStart(3, " ")}: ${label} - ${address}\r\n`
          );
        }
        const input = await this.socketInput();
        const num = parseInt(input, 10);
        const choice = addressBook[num];
        if (choice) {
          await this.bridgeConnection(choice);
        } else {
          socket.write(`Invalid choice ${input}\r\n`);
        }
      } catch (err) {
        if (err.message !== "disconnected") {
          log.error(err);
        }
        break;
      }
    }
  }

  async bridgeConnection(choice) {
    const [label, address] = choice;
    const [host, port = "23"] = address.split(":");

    this.socket.write(`Connecting to ${label} - ${address}...\r\n\r\n`);

    const client = net.connect(port, host, () => {
      this.log.info({
        msg: "bridgeConnected",
        label,
        address,
      });
    });

    client.on("data", (data) => this.socket.write(data));
    const handleInput = (data) => client.write(data);
    this.socket.on("data", handleInput);

    return new Promise((resolve, reject) => {
      client.on("end", () => {
        this.log.info({
          msg: "bridgeDisconnected",
          label,
          address,
        });
        this.socket.removeListener("data", handleInput);
        this.socket.write(`\r\n\r\nDisconnected from ${label} - ${address}\r\n\r\n`);
        resolve();
      });
      client.on("error", (error) => reject(error));
    });
  }

  async socketInput(prompt = "> ") {
    return new Promise((resolve, reject) => {
      this.socket.write(prompt);
      this.socket.once("data", (data) => resolve(data.toString("utf8").trim()));
      this.socket.once("end", () => reject(new Error("disconnected")));
    });
  }
}

main().catch(console.error);
