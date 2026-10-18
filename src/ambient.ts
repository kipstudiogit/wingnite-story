class AmbientAudioEngine {
  private ctx: AudioContext | null = null;
  private rainNode: AudioNode | null = null;
  private fireNode: AudioNode | null = null;
  private windNode: AudioNode | null = null;
  private windLfo: OscillatorNode | null = null;
  private rainGain: GainNode | null = null;
  private fireGain: GainNode | null = null;
  private windGain: GainNode | null = null;

  private initContext() {
    if (!this.ctx) {
      const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      this.ctx = new AudioCtx();
    }
    if (this.ctx.state === "suspended") {
      this.ctx.resume();
    }
  }

  public toggleRain(enable: boolean, volume = 0.25) {
    this.initContext();
    if (!this.ctx) return;

    if (!enable) {
      if (this.rainNode) {
        this.rainNode.disconnect();
        this.rainNode = null;
      }
      return;
    }

    if (this.rainNode) return;

    const bufferSize = this.ctx.sampleRate * 2;
    const buffer = this.ctx.createBuffer(1, bufferSize, this.ctx.sampleRate);
    const output = buffer.getChannelData(0);
    for (let i = 0; i < bufferSize; i++) {
      output[i] = Math.random() * 2 - 1;
    }

    const whiteNoise = this.ctx.createBufferSource();
    whiteNoise.buffer = buffer;
    whiteNoise.loop = true;

    const filter = this.ctx.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.setValueAtTime(800, this.ctx.currentTime);

    this.rainGain = this.ctx.createGain();
    this.rainGain.gain.setValueAtTime(volume, this.ctx.currentTime);

    whiteNoise.connect(filter);
    filter.connect(this.rainGain);
    this.rainGain.connect(this.ctx.destination);
    whiteNoise.start(0);
    this.rainNode = whiteNoise;
  }

  public toggleFire(enable: boolean, volume = 0.3) {
    this.initContext();
    if (!this.ctx) return;

    if (!enable) {
      if (this.fireNode) {
        this.fireNode.disconnect();
        this.fireNode = null;
      }
      return;
    }

    if (this.fireNode) return;

    const bufferSize = this.ctx.sampleRate * 2;
    const buffer = this.ctx.createBuffer(1, bufferSize, this.ctx.sampleRate);
    const output = buffer.getChannelData(0);
    let lastOut = 0.0;
    for (let i = 0; i < bufferSize; i++) {
      const brown = (lastOut + 0.02 * (Math.random() * 2 - 1)) / 1.02;
      output[i] = brown * 3.5;
      lastOut = brown;
      if (Math.random() < 0.002) {
        output[i] += (Math.random() * 2 - 1) * 0.8;
      }
    }

    const noise = this.ctx.createBufferSource();
    noise.buffer = buffer;
    noise.loop = true;

    const filter = this.ctx.createBiquadFilter();
    filter.type = "bandpass";
    filter.frequency.setValueAtTime(1200, this.ctx.currentTime);
    filter.Q.setValueAtTime(1.5, this.ctx.currentTime);

    this.fireGain = this.ctx.createGain();
    this.fireGain.gain.setValueAtTime(volume, this.ctx.currentTime);

    noise.connect(filter);
    filter.connect(this.fireGain);
    this.fireGain.connect(this.ctx.destination);
    noise.start(0);
    this.fireNode = noise;
  }

  public toggleWind(enable: boolean, volume = 0.2) {
    this.initContext();
    if (!this.ctx) return;

    if (!enable) {
      if (this.windNode) {
        this.windNode.disconnect();
        this.windNode = null;
      }
      if (this.windLfo) {
        this.windLfo.stop();
        this.windLfo.disconnect();
        this.windLfo = null;
      }
      return;
    }

    if (this.windNode) return;

    const bufferSize = this.ctx.sampleRate * 2;
    const buffer = this.ctx.createBuffer(1, bufferSize, this.ctx.sampleRate);
    const output = buffer.getChannelData(0);
    for (let i = 0; i < bufferSize; i++) {
      output[i] = Math.random() * 2 - 1;
    }

    const noise = this.ctx.createBufferSource();
    noise.buffer = buffer;
    noise.loop = true;

    const filter = this.ctx.createBiquadFilter();
    filter.type = "bandpass";
    filter.frequency.setValueAtTime(400, this.ctx.currentTime);
    filter.Q.setValueAtTime(3.0, this.ctx.currentTime);

    const lfo = this.ctx.createOscillator();
    lfo.frequency.setValueAtTime(0.2, this.ctx.currentTime);

    const lfoGain = this.ctx.createGain();
    lfoGain.gain.setValueAtTime(250, this.ctx.currentTime);

    lfo.connect(lfoGain);
    lfoGain.connect(filter.frequency);

    this.windGain = this.ctx.createGain();
    this.windGain.gain.setValueAtTime(volume, this.ctx.currentTime);

    noise.connect(filter);
    filter.connect(this.windGain);
    this.windGain.connect(this.ctx.destination);

    noise.start(0);
    lfo.start(0);
    this.windNode = noise;
    this.windLfo = lfo;
  }

  public stopAll() {
    this.toggleRain(false);
    this.toggleFire(false);
    this.toggleWind(false);
  }
}

export const ambientEngine = new AmbientAudioEngine();