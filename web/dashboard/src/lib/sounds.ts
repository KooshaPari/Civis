let audioContext: AudioContext | null = null;

function getAudioContext() {
  if (typeof window === "undefined") return null;
  const Ctor = window.AudioContext || (window as Window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Ctor) return null;
  if (!audioContext) audioContext = new Ctor();
  return audioContext;
}

async function ensureRunning() {
  const ctx = getAudioContext();
  if (!ctx) return null;
  if (ctx.state === "suspended") {
    try {
      await ctx.resume();
    } catch {
      return null;
    }
  }
  return ctx;
}

function stopAt(time: number, ...nodes: Array<AudioNode | null | undefined>) {
  for (const node of nodes) {
    if (!node) continue;
    try {
      if ("stop" in node) (node as AudioScheduledSourceNode).stop(time);
    } catch {
      /* ignored */
    }
  }
}

function playTone(frequencies: number[], durationMs: number, type: OscillatorType = "sine") {
  void (async () => {
    const ctx = await ensureRunning();
    if (!ctx) return;
    const gain = ctx.createGain();
    gain.gain.value = 0.0001;
    gain.connect(ctx.destination);
    const now = ctx.currentTime;
    const end = now + durationMs / 1000;
    gain.gain.exponentialRampToValueAtTime(0.12, now + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, end);
    const oscillators = frequencies.map((frequency) => {
      const osc = ctx.createOscillator();
      osc.type = type;
      osc.frequency.value = frequency;
      osc.connect(gain);
      osc.start(now);
      osc.stop(end);
      return osc;
    });
    stopAt(end + 0.05, gain, ...oscillators);
  })();
}

function playNoise(durationMs: number) {
  void (async () => {
    const ctx = await ensureRunning();
    if (!ctx) return;
    const buffer = ctx.createBuffer(1, Math.max(1, Math.floor(ctx.sampleRate * (durationMs / 1000))), ctx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < data.length; i += 1) {
      data[i] = Math.random() * 2 - 1;
    }
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    const gain = ctx.createGain();
    gain.gain.value = 0.05;
    source.connect(gain);
    gain.connect(ctx.destination);
    const now = ctx.currentTime;
    source.start(now);
    source.stop(now + durationMs / 1000);
    stopAt(now + durationMs / 1000 + 0.05, gain, source);
  })();
}

export function playBirth() { playTone([440, 660, 880], 100, "sine"); }
export function playDeath() { playTone([440, 330, 220], 150, "sine"); }
export function playConflict() { playNoise(200); }
export function playTech() { playTone([523, 1046], 300, "sine"); }
export function playDisaster() { playTone([80], 500, "sine"); }
export function playClick() { playTone([1000], 30, "sine"); }

export async function primeAudio() {
  await ensureRunning();
}
