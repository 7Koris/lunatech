<!-- [![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Unlicense License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url] -->

<br />
<div align="center">
  <h3 align="center">LunaTech</h3>
  <p align="center">
    a solution for powering realtime audio visualizations
  </p>
</div>

<!-- ABOUT THE PROJECT -->

## About The Project

LunaTech is a server-based solution that analyzes system audio in realtime and broadcasts the results over the network to clients that can read **OSC** (open sound control) packets. LunaTech is designed in this manner such that multiple applications on a given local network can rely on this data to provide realtime reactive visualizations.

- An audio/visual setup might contain both physical lights and a projector setup, requiring synchronization between components.
- A broadcasting approach simplifies your A/V setup.
- The server is built in rust to provide a reliable backend for live visualizations.

## Getting Started

LunaTech has thus far only been tested on Linux. Windows support is planned.

### Installation

1. Go to the releases page and download the latest release for your system.

## Usage

LunaTech can be used by both the commandline and graphical user interface. The commandline provides some additional options not available in the GUI mode as well.

```md
Realtime audio analysis server

Usage: lt_server [OPTIONS]

Options:
  -r, --sample_rate <sample_rate>  Sets the sample rate
  -b, --buffer_size <buffer_size>  Sets the buffer size
  -p, --port <port>                Set the port to broadcast on
  -H, --HEADLESS                   Enable headless mode; server starts by default
  -I, --I                          Monitor input device instead of output device
  -h, --help                       Print help
  -V, --version                    Print version
```

When running in GUI mode, simply click the large circular button to start the server. Settings that are changed will be applied when you click the "Update Settings" button.

![GUI](image.png)

### OSC Addresses
- /lt/broad_range_rms
- /lt/low_range_rms
- /lt/mid_range_rms
- /lt/high_range_rms
- /lt/zcr
- /lt/spectral_centroid
- /lt/flux

## Roadmap

- [x] Basic audio analysis
- [x] OSC broadcasting 
- [x] Basic GUI
- [ ] Tempo Prediction
- [ ] Automatic Gain Correction
- [ ] Improved Github page and Developer docs
- [ ] Windows support

## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

## Acknowledgments

Use this space to list resources you find helpful and would like to give credit to. I've included a few of my favorites to kick things off!

- [The rust audio community](https://rust.audio/)
- [More About FFTs](https://www.ap.com/news/more-about-ffts)
- [Magnitude and Phase Spectra](https://pages.jh.edu/signals/spectra/spectra.html)
- [Understanding the FFT Algorithm](https://jakevdp.github.io/blog/2013/08/28/understanding-the-fft/)