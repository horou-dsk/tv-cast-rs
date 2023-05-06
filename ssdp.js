const dgram = require('dgram');
const udp4 = dgram.createSocket('udp4');

const PORT = 45233;
const HOST = '0.0.0.0';

const to_addr = '192.169.1.19';

udp4.on('message', (msg, rinfo) => {
  let result = msg.toString();
  if (rinfo.address === to_addr && !result.startsWith('NOTIFY')) {
    console.log(`server got: ${result} from ${rinfo.address}:${rinfo.port}`);
  }
});

udp4.on('listening', () => {
  const address = udp4.address();
  console.log(`server listening ${address.address}:${address.port}`);
});

udp4.bind(PORT, () => {
  udp4.addMembership('239.255.255.250', '192.169.1.28');
  udp4.setMulticastLoopback(false);
});

const notify = `M-SEARCH * HTTP/1.1
MX: 15
MAN: "ssdp:discover"
HOST: 239.255.255.250:1900
ST: urn:schemas-upnp-org:device:MediaRenderer:1

`

setInterval(() => {
  udp4.send(notify, 1900, to_addr, (error) => {
    console.log(`${error} send ok`);
  });
}, 3000);
