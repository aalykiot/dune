/**
 * Domain Name System (DNS) APIs
 *
 * Although named for the Domain Name System (DNS), it does not always use the
 * DNS protocol for lookups. This module uses the operating system facilities
 * to perform name resolution. It may not need to perform any network communication.
 *
 * @see {@link https://nodejs.org/api/dns.html#dns}
 *
 * @module DNS
 */

const binding = process.binding('dns');

/**
 * @constant {RegExp} - A regular expression that matches IPv4 addresses.
 */
export const IP_ADDRESS_V4 = new RegExp(
  '(([0-9]|[1-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])'
);

/**
 * @constant {RegExp} - A regular expression that matches IPv6 addresses.
 */
export const IP_ADDRESS_V6 = new RegExp(
  '((([0-9a-fA-F]){1,4})\\:){7}([0-9a-fA-F]){1,4}'
);

/**
 * @typedef {Object} Resolution
 * @property {string} address - A string representation of an IP address.
 * @property {string} family - Denoting the family of the address (`IPv4` or `IPv6`).
 */

/**
 * Resolves a host name into the first found A (IPv4) or AAAA (IPv6) record.
 *
 * @param {String} hostname - Host name to resolve.
 * @returns {Promise<Resolution[]>} An array of resolved hostnames.
 */
export async function lookup(hostname) {
  // Check the data argument type.
  if (!hostname || typeof hostname !== 'string') {
    throw new TypeError(`The "hostname" argument must be of type string.`);
  }

  // Check if the hostname is already an IPv4 address.
  if (IP_ADDRESS_V4.test(hostname)) {
    return [{ family: 'IPv4', address: hostname }];
  }

  // Check if the hostname is already an IPv6 address.
  if (IP_ADDRESS_V6.test(hostname)) {
    return [{ family: 'IPv6', address: hostname }];
  }

  return binding.lookup(hostname);
}

export default {
  lookup,
};
