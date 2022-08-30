// Domain Name System (DNS) APIs
//
// Although named for the Domain Name System (DNS), it does not always use the
// DNS protocol for lookups. This module uses the operating system facilities
// to perform name resolution. It may not need to perform any network communication.
//
// https://nodejs.org/api/dns.html#dns

const binding = process.binding('dns');

/**
 * Resolves a host name into the first found A (IPv4) or AAAA (IPv6) record.
 *
 * @param {String} hostname
 * @returns {Promise<Array<{String, String}>}
 */
export async function lookup(hostname) {
  // Check the data argument type.
  if (!hostname || typeof hostname !== 'string') {
    throw new TypeError(`The "hostname" argument must be of type string.`);
  }

  return binding.lookup(hostname);
}

export default {
  lookup,
};
