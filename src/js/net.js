// TCP Networking APIs
//
// The TCP Networking APIs provide an asynchronous network API for creating
// stream-based TCP servers and clients.
//
// https://nodejs.org/dist/latest-v18.x/docs/api/net.html

import dns from 'dns';
import { EventEmitter } from 'events';

const binding = process.binding('net');

/**
 * A Socket object is a wrapper for a raw TCP socket.
 */
export class Socket extends EventEmitter {
  /**
   * Creates a new Socket instance.
   *
   * @returns {Socket}
   */
  constructor() {
    super();
    this._rid = undefined;
    this._connecting = false;
    this._encoding = undefined;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
  }

  /**
   * Initiates a connection on a given remote host.
   *
   * @param {Object} options
   * @param {Function} onConnection
   */
  async connect(options = {}, onConnection) {
    // Check if socket is already connected.
    if (this._rid) {
      this.emit('error', new Error(`Socket is already connected.`));
      return;
    }

    // Check if a connection is happening.
    if (this._connecting) {
      this.emit('error', new Error(`Socket is trying to connect.`));
      return;
    }

    // Subscribe to the emitter the on-connect callback if specified.
    if (onConnection) this.on('connect', onConnection);

    // Parse provided options.
    const hostname = options?.host || '127.0.0.1';
    const port = options?.port || 80;

    this._connecting = true;

    try {
      // Use DNS lookup to resolve the hostname.
      const addresses = await dns.lookup(hostname);

      // Prefer IPv4 address.
      const host = addresses.some((addr) => addr.family === 'IPv4')
        ? addresses.filter((addr) => addr.family === 'IPv4')[0].address
        : addresses[0].address;

      // Try to connect to the remote host.
      const socketInfo = await binding.connect(host, port);

      // Update socket's local state.
      this._rid = socketInfo.rid;
      this.remoteAddress = socketInfo.remoteAddress;
      this.remotePort = socketInfo.remotePort;
      this._connecting = false;

      // Fire the success event.
      this.emit('connect', socketInfo);
    } catch (e) {
      this.emit('error', e);
    }
  }
}

/**
 * Initiates a connection on a given remote host.
 *
 * @param {Object} options
 * @param {Function} onConnection
 */
export function createConnection(options, onConnection) {
  const socket = new Socket();
  socket.connect(options, onConnection);
  return socket;
}

export default {
  Socket,
  createConnection,
};
