#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <functional>
#include <numbers>
#include <optional>
#include <random>
#include <string>
#include <vector>

#include "audio_dsp.hpp"

using namespace qs::windows::recorder;

namespace {

int failures = 0;

void check(bool ok, const std::string& name, const std::string& detail = {}) {
	std::printf("%s  %s%s%s\n", ok ? "PASS" : "FAIL", name.c_str(), detail.empty() ? "" : "  ", detail.c_str());
	if (!ok) failures++;
}

std::string fmt(const char* format, double a, double b = 0, double c = 0, double d = 0) {
	char buf[256];
	std::snprintf(buf, sizeof(buf), format, a, b, c, d);
	return buf;
}

constexpr double TAU = 2.0 * std::numbers::pi;

struct Phase {
	double amplitude = 0;
	double offset = 0;
};

Phase sinePhase(const std::vector<float>& stereo, std::size_t from, std::size_t count, double freq, int rate, int channel = 0) {
	double i = 0;
	double q = 0;
	for (std::size_t n = from; n < from + count; ++n) {
		auto t = static_cast<double>(n) / rate;
		auto v = static_cast<double>(stereo[n * 2 + static_cast<std::size_t>(channel)]);
		i += v * std::sin(TAU * freq * t);
		q += v * std::cos(TAU * freq * t);
	}
	i *= 2.0 / static_cast<double>(count);
	q *= 2.0 / static_cast<double>(count);
	return {std::hypot(i, q), std::atan2(q, i) / (TAU * freq)};
}

void testWeights() {
	auto mono = stereoWeights(1, 0);
	check(mono.size() == 1 && mono[0][0] == 1.0F && mono[0][1] == 1.0F, "mono weights duplicate to both channels");

	auto stereo = stereoWeights(2, 0x3);
	check(stereo[0][0] == 1 && stereo[0][1] == 0 && stereo[1][0] == 0 && stereo[1][1] == 1, "stereo weights are identity");

	auto surround = stereoWeights(6, 0x3F);
	float left = 0;
	float right = 0;
	for (const auto& w: surround) {
		left += w[0];
		right += w[1];
	}
	check(left <= 1.0001F && right <= 1.0001F && surround[3][0] == 0 && surround[3][1] == 0,
	      "5.1 fold keeps a full scale signal on every channel at or below full scale",
	      fmt("sums %.4f %.4f", left, right));

	auto unmasked = stereoWeights(4, 0);
	check(unmasked[0][0] == 1 && unmasked[1][1] == 1 && unmasked[2][0] == 0 && unmasked[3][1] == 0,
	      "unmasked multichannel takes the first two channels");
}

void testConvert() {
	std::vector<std::int16_t> mono = {0, 16384, -32768, 32767};
	std::vector<float> out;
	convertToStereo(reinterpret_cast<const std::uint8_t*>(mono.data()), mono.size(), SampleType::Int16, 2, stereoWeights(1, 0), out);
	auto ok = out.size() == 8;
	for (std::size_t i = 0; ok && i < mono.size(); ++i) {
		ok = out[i * 2] == out[i * 2 + 1] && std::abs(out[i * 2] - mono[i] / 32768.0F) < 1e-6F;
	}
	check(ok, "mono int16 microphone becomes identical left and right");

	std::vector<std::uint8_t> s24 = {0x00, 0x00, 0x40, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x80, 0x01, 0x00, 0x00};
	convertToStereo(s24.data(), 2, SampleType::Int24, 3, stereoWeights(2, 0), out);
	check(std::abs(out[0] - 0.5F) < 1e-6F && std::abs(out[1] + 1.0F / 8388608.0F) < 1e-9F && std::abs(out[2] + 1.0F) < 1e-6F
	          && std::abs(out[3] - 1.0F / 8388608.0F) < 1e-9F,
	      "24-bit samples decode with sign");

	std::vector<float> f32 = {0.25F, -0.5F, 0.75F, 1.5F};
	convertToStereo(reinterpret_cast<const std::uint8_t*>(f32.data()), 2, SampleType::Float32, 4, stereoWeights(2, 0x3), out);
	check(out == f32, "float stereo passes through unchanged");

	convertToStereo(nullptr, 3, SampleType::Float32, 4, stereoWeights(2, 0x3), out);
	check(out.size() == 6 && std::ranges::all_of(out, [](float v) { return v == 0; }), "silent packets become zeros");
}

double resampleError(int inRate, int outRate, double freq, double correction, std::size_t chunk) {
	Resampler r;
	r.configure(inRate, outRate);
	r.setCorrection(correction);
	auto step = r.step();

	std::vector<float> in;
	auto seconds = 2.0;
	auto frames = static_cast<std::size_t>(inRate * seconds);
	in.resize(frames * 2);
	for (std::size_t n = 0; n < frames; ++n) {
		auto v = static_cast<float>(0.5 * std::sin(TAU * freq * n / inRate));
		in[n * 2] = v;
		in[n * 2 + 1] = -v;
	}

	std::vector<float> out;
	for (std::size_t n = 0; n < frames; n += chunk) r.process(in.data() + n * 2, std::min(chunk, frames - n), out);

	double worst = 0;
	auto count = out.size() / 2;
	for (std::size_t n = 64; n + 64 < count; ++n) {
		auto position = static_cast<double>(n) * step;
		auto ideal = 0.5 * std::sin(TAU * freq * position / inRate);
		worst = std::max(worst, std::abs(out[n * 2] - ideal));
		worst = std::max(worst, std::abs(out[n * 2 + 1] + ideal));
	}
	return worst;
}

void testResampler() {
	struct Case {
		int in, out;
		double freq, correction;
		double limit;
	};
	const Case cases[] = {
	    {48000, 48000, 1000, 0.0, 1e-3},
	    {48000, 48000, 1000, 0.0031, 1e-3},
	    {48000, 48000, 9000, -0.0047, 1e-3},
	    {44100, 48000, 1000, 0.0, 1e-3},
	    {44100, 48000, 7000, 0.002, 1e-3},
	    {16000, 48000, 440, 0.0, 1e-3},
	    {16000, 48000, 3000, -0.003, 1e-3},
	    {96000, 48000, 1000, 0.0, 1e-3},
	    {192000, 48000, 5000, 0.001, 1e-3},
	    {8000, 44100, 300, 0.0, 1e-3},
	};

	for (const auto& c: cases) {
		auto error = resampleError(c.in, c.out, c.freq, c.correction, 441);
		check(error < c.limit,
		      fmt("resample %.0f -> %.0f Hz, %.0f Hz tone, correction %+.4f", c.in, c.out, c.freq, c.correction),
		      fmt("max error %.2e (%.1f dB)", error, 20 * std::log10(error / 0.5)));
	}

	Resampler r;
	r.configure(96000, 48000);
	std::vector<float> in(96000 * 2);
	for (std::size_t n = 0; n < 96000; ++n) in[n * 2] = in[n * 2 + 1] = static_cast<float>(0.5 * std::sin(TAU * 30000 * n / 96000.0));
	std::vector<float> out;
	r.process(in.data(), 96000, out);
	double energy = 0;
	for (std::size_t n = 200; n < out.size() / 2 - 200; ++n) energy += out[n * 2] * out[n * 2];
	auto rms = std::sqrt(energy / static_cast<double>(out.size() / 2 - 400));
	check(rms < 0.5 / std::sqrt(2.0) * 0.01, "downsampling 96 kHz rejects a 30 kHz tone (no alias at 18 kHz)", fmt("rms %.2e (%.1f dB)", rms, 20 * std::log10(rms / (0.5 / std::sqrt(2.0)))));

	r.configure(48000, 48000);
	std::vector<float> dc(4800 * 2, 0.25F);
	out.clear();
	r.process(dc.data(), 4800, out);
	auto dcOk = true;
	for (std::size_t n = 32; n < out.size() / 2; ++n) dcOk = dcOk && std::abs(out[n * 2] - 0.25F) < 1e-5F;
	check(dcOk, "resampler has unity gain at DC");
}

struct Device {
	int nominalRate = 48000;
	double ppm = 0;
	std::size_t packet = 480;
	double jitterMs = 0;
	double startSeconds = 0.02;
	double latencySeconds = 0.01;
	double freq = 440;
	double amplitude = 0.5;
	bool mono = false;
};

struct Simulation {
	std::vector<float> out;
	AlignedStream stream;
	std::int64_t snapsAfterStart = 0;
	double maxErrorFramesLate = 0;
	double finalCorrection = 0;
	double meanCorrection = 0;
	double wobble = 0;
	double phaseOffsetMs = 0;
	double amplitude = 0;
	double maxSecondDiff = 0;
};

using Gate = std::function<bool(double)>;

Simulation simulate(const Device& d, int outRate, double seconds, double margin, const Gate& delivering = {}) {
	Simulation sim;
	sim.stream.configure(d.nominalRate, outRate);
	auto trueRate = d.nominalRate * (1.0 + d.ppm * 1e-6);
	std::mt19937 rng(1234);
	std::normal_distribution<double> jitter(0.0, d.jitterMs / 1000.0);

	std::vector<float> packet(d.packet * 2);
	std::vector<float> produced;
	std::uint64_t frameIndex = 0;
	auto nextPacketTime = d.startSeconds;
	auto snapsBefore = std::int64_t {0};
	double correctionSum = 0;
	double correctionSquares = 0;
	double correctionCount = 0;

	for (double now = 0; now < seconds; now += 0.01) {
		while (nextPacketTime + d.packet / trueRate + d.latencySeconds <= now) {
			auto start = nextPacketTime;
			auto on = !delivering || delivering(start);
			if (on) {
				for (std::size_t k = 0; k < d.packet; ++k) {
					auto t = start + static_cast<double>(k) / trueRate;
					auto v = static_cast<float>(d.amplitude * std::sin(TAU * d.freq * t));
					packet[k * 2] = v;
					packet[k * 2 + 1] = d.mono ? v : -v;
				}
				produced.clear();
				auto stamp = start + jitter(rng);
				sim.stream.push(packet.data(), d.packet, stamp * outRate, produced);
				sim.out.insert(sim.out.end(), produced.begin(), produced.end());
				if (now > 2.0) sim.maxErrorFramesLate = std::max(sim.maxErrorFramesLate, std::abs(sim.stream.lastError()));
			}
			frameIndex += d.packet;
			nextPacketTime = d.startSeconds + static_cast<double>(frameIndex) / trueRate;
		}

		produced.clear();
		sim.stream.padTo(static_cast<std::int64_t>((now - margin) * outRate), produced);
		sim.out.insert(sim.out.end(), produced.begin(), produced.end());
		if (now < 1.0) snapsBefore = sim.stream.snaps();
		if (now > seconds / 3) {
			correctionSum += sim.stream.correction();
			correctionSquares += sim.stream.correction() * sim.stream.correction();
			correctionCount += 1;
		}
	}

	if (correctionCount > 0) {
		sim.meanCorrection = correctionSum / correctionCount;
		sim.wobble = std::sqrt(std::max(0.0, correctionSquares / correctionCount - sim.meanCorrection * sim.meanCorrection));
	}

	sim.snapsAfterStart = sim.stream.snaps() - snapsBefore;
	sim.finalCorrection = sim.stream.correction();

	auto frames = sim.out.size() / 2;
	auto window = static_cast<std::size_t>(outRate);
	if (frames > window * 2) {
		auto p = sinePhase(sim.out, frames - window - outRate / 10, window, d.freq, outRate);
		sim.phaseOffsetMs = p.offset * 1000.0;
		sim.amplitude = p.amplitude;
	}

	auto from = static_cast<std::size_t>((d.startSeconds + 0.2) * outRate);
	for (std::size_t n = from + 1; n + 1 < frames; ++n) {
		auto a = sim.out[(n - 1) * 2];
		auto b = sim.out[n * 2];
		auto c = sim.out[(n + 1) * 2];
		sim.maxSecondDiff = std::max(sim.maxSecondDiff, static_cast<double>(std::abs(a - 2 * b + c)));
	}

	return sim;
}

void testDrift() {
	struct Case {
		const char* name;
		Device device;
		int outRate;
	};
	const Case cases[] = {
	    {"loopback 48 kHz, +120 ppm", {48000, 120, 480, 0.2}, 48000},
	    {"loopback 48 kHz, -350 ppm", {48000, -350, 480, 0.2}, 48000},
	    {"loopback 44.1 kHz, +60 ppm", {44100, 60, 441, 0.2}, 44100},
	    {"microphone 48 kHz, -900 ppm, 1 ms jitter", {48000, -900, 480, 1.0}, 48000},
	    {"microphone 16 kHz mono, +1500 ppm", {16000, 1500, 160, 0.5, 0.02, 0.01, 440, 0.5, true}, 48000},
	    {"microphone 44.1 kHz into 48 kHz, +400 ppm", {44100, 400, 441, 0.3}, 48000},
	    {"USB microphone, 2 ms packets, -200 ppm", {48000, -200, 96, 0.5}, 48000},
	    {"Bluetooth microphone, 60 ms packets, +250 ppm", {16000, 250, 960, 2.0, 0.02, 0.06}, 48000},
	};

	for (const auto& c: cases) {
		auto sim = simulate(c.device, c.outRate, 300.0, 0.2);
		auto expected = static_cast<double>(c.device.ppm) * 1e-6;
		auto slope = 2 * std::numbers::pi * c.device.freq / c.outRate;
		auto smoothLimit = c.device.amplitude * slope * slope * 1.5 + 0.002;
		auto wobbleLimit = c.device.jitterMs * 2.5e-4 + 2e-5;
		auto ok = sim.snapsAfterStart == 0 && std::abs(sim.phaseOffsetMs) < 1.0
		       && std::abs(sim.meanCorrection - expected) < 3e-5 && sim.wobble < wobbleLimit
		       && std::abs(sim.amplitude - c.device.amplitude) < 0.01 && sim.maxSecondDiff < smoothLimit;
		check(ok,
		      std::string("drift: ") + c.name,
		      fmt("offset %.3f ms, correction %+.0f ppm (wobble %.0f ppm), snaps %.0f", sim.phaseOffsetMs, sim.meanCorrection * 1e6, sim.wobble * 1e6, static_cast<double>(sim.snapsAfterStart))
		          + fmt(", worst error %.1f frames, 2nd diff %.2e (limit %.2e)", sim.maxErrorFramesLate, sim.maxSecondDiff, smoothLimit));
	}

	auto sim = simulate({48000, 8000, 480, 0.2}, 48000, 120.0, 0.2);
	check(std::abs(sim.phaseOffsetMs) < 60.0 && sim.snapsAfterStart > 0,
	      "drift beyond the correction range snaps instead of drifting away",
	      fmt("offset %.3f ms, snaps %.0f", sim.phaseOffsetMs, static_cast<double>(sim.snapsAfterStart)));
}

void testGaps() {
	Device d {48000, 150, 480, 0.0};
	auto gate = [](double t) { return t < 2.0 || t >= 5.0; };
	auto sim = simulate(d, 48000, 12.0, 0.1, gate);

	auto silent = true;
	for (auto n = static_cast<std::size_t>(2.01 * 48000); n < static_cast<std::size_t>(4.99 * 48000); ++n) {
		silent = silent && sim.out[n * 2] == 0.0F && sim.out[n * 2 + 1] == 0.0F;
	}

	std::size_t firstSound = 0;
	for (auto n = static_cast<std::size_t>(4.5 * 48000); n < sim.out.size() / 2; ++n) {
		if (sim.out[n * 2] != 0.0F) {
			firstSound = n;
			break;
		}
	}

	auto trueRate = 48000 * (1 + 150e-6);
	auto resumeFrame = std::ceil((5.0 - d.startSeconds) * trueRate / 480.0) * 480.0;
	auto resumeTime = d.startSeconds + resumeFrame / trueRate;
	auto expectedFirst = resumeTime * 48000;
	auto p = sinePhase(sim.out, static_cast<std::size_t>(6.0 * 48000), 48000, d.freq, 48000);
	auto before = sinePhase(sim.out, static_cast<std::size_t>(0.5 * 48000), 48000, d.freq, 48000);

	check(silent, "a pause in playback is filled with silence");
	check(std::abs(static_cast<double>(firstSound) - expectedFirst) < 3.0,
	      "sound after the pause lands where its timestamp says",
	      fmt("first sample %.0f, expected %.1f", static_cast<double>(firstSound), expectedFirst));
	check(std::abs(p.offset - before.offset) * 1000 < 0.5 && std::abs(p.offset) * 1000 < 0.5,
	      "the stream stays in phase with the clock across the pause",
	      fmt("before %.3f ms, after %.3f ms", before.offset * 1000, p.offset * 1000));
	check(sim.stream.written() >= static_cast<std::int64_t>((12.0 - 0.1 - 0.02) * 48000),
	      "the stream keeps up with the clock",
	      fmt("written %.0f", static_cast<double>(sim.stream.written())));

	Device mic {48000, 0, 480, 0.2};
	auto lost = [](double t) { return t < 3.0; };
	auto lostSim = simulate(mic, 48000, 10.0, 0.2, lost);
	auto tailSilent = true;
	for (auto n = static_cast<std::size_t>(3.05 * 48000); n < lostSim.out.size() / 2; ++n) tailSilent = tailSilent && lostSim.out[n * 2] == 0.0F;
	check(tailSilent && lostSim.stream.written() >= static_cast<std::int64_t>(9.79 * 48000),
	      "a device lost mid-recording leaves silence and the stream goes on",
	      fmt("written %.0f", static_cast<double>(lostSim.stream.written())));
}

void testOverlap() {
	AlignedStream stream;
	stream.configure(48000, 48000);
	std::vector<float> ramp(4800 * 2);
	for (std::size_t n = 0; n < 4800; ++n) ramp[n * 2] = ramp[n * 2 + 1] = static_cast<float>(n) / 48000.0F;

	std::vector<float> out;
	stream.push(ramp.data(), 4800, -2400.0, out);
	auto ok = out.size() >= 400;
	for (std::size_t n = 0; ok && n < 200; ++n) ok = std::abs(out[n * 2] - static_cast<float>(2400 + n) / 48000.0F) < 1e-4F;
	check(ok, "audio captured before the recording started is trimmed", fmt("first %.6f, want %.6f", out.empty() ? -1.0 : out[0], 2400.0 / 48000));

	AlignedStream late;
	late.configure(48000, 48000);
	std::vector<float> sine(48000 * 2);
	for (std::size_t n = 0; n < 48000; ++n) sine[n * 2] = sine[n * 2 + 1] = static_cast<float>(0.5 * std::sin(TAU * 997 * n / 48000.0));
	out.clear();
	late.push(sine.data(), 48000, 1000.25, out);
	auto lead = true;
	for (std::size_t n = 0; n < 1001; ++n) lead = lead && out[n * 2] == 0.0F;
	double worst = 0;
	for (std::size_t n = 1100; n < 40000; ++n) {
		auto ideal = 0.5 * std::sin(TAU * 997 * (static_cast<double>(n) - 1000.25) / 48000.0);
		worst = std::max(worst, std::abs(out[n * 2] - ideal));
	}
	check(lead && worst < 1e-4, "the first packet is placed with sub-sample accuracy", fmt("max error %.2e", worst));

	AlignedStream noStamp;
	noStamp.configure(48000, 48000);
	out.clear();
	noStamp.push(ramp.data(), 4800, std::nullopt, out);
	noStamp.push(ramp.data(), 4800, std::nullopt, out);
	check(noStamp.written() >= 9600 - 12 && noStamp.written() <= 9600, "packets without a timestamp are appended back to back");
}

std::vector<std::int16_t> mixAll(AudioMixer& mixer, bool drain) {
	std::vector<std::int16_t> out;
	mixer.mix(drain, out);
	return out;
}

void testMixer() {
	{
		FrameQueue a;
		FrameQueue b;
		AudioMixer mixer(48000);
		mixer.addSource(&a);
		mixer.addSource(&b);

		std::vector<float> sa(48000 * 2);
		std::vector<float> sb(48000 * 2);
		for (std::size_t n = 0; n < 48000; ++n) {
			sa[n * 2] = sa[n * 2 + 1] = static_cast<float>(0.3 * std::sin(TAU * 440 * n / 48000.0));
			sb[n * 2] = static_cast<float>(0.3 * std::sin(TAU * 1250 * n / 48000.0));
			sb[n * 2 + 1] = -sb[n * 2];
		}
		a.append(sa);
		b.append(sb);

		std::vector<std::int16_t> out;
		auto part = mixAll(mixer, false);
		auto rest = mixAll(mixer, true);
		out.insert(out.end(), part.begin(), part.end());
		out.insert(out.end(), rest.begin(), rest.end());

		auto exact = out.size() == sa.size();
		for (std::size_t i = 0; exact && i < out.size(); ++i) {
			auto want = static_cast<std::int16_t>(std::lrintf((sa[i] + sb[i]) * 32767.0F));
			exact = std::abs(out[i] - want) <= 1;
		}
		check(exact, "quiet sources are summed unchanged and without delay");
	}

	{
		FrameQueue a;
		FrameQueue b;
		AudioMixer mixer(48000);
		mixer.addSource(&a);
		mixer.addSource(&b);

		std::vector<float> loud(48000 * 2);
		for (std::size_t n = 0; n < 48000; ++n) loud[n * 2] = loud[n * 2 + 1] = static_cast<float>(0.8 * std::sin(TAU * 300 * n / 48000.0));
		a.append(loud);
		b.append(loud);

		auto out = mixAll(mixer, true);
		auto peak = 0;
		for (auto v: out) peak = std::max(peak, std::abs(static_cast<int>(v)));
		auto ceiling = static_cast<int>(std::lrint(PeakLimiter::THRESHOLD * 32767.0)) + 1;
		check(out.size() == loud.size() && peak <= ceiling, "two loud sources never clip", fmt("peak %.0f, ceiling %.0f", peak, ceiling));

		double energy = 0;
		for (std::size_t n = 0; n < out.size(); n += 2) energy += (out[n] / 32767.0) * (out[n] / 32767.0);
		auto rms = std::sqrt(energy / static_cast<double>(out.size() / 2));
		check(rms > 0.55, "the limiter only takes the peaks down", fmt("rms %.3f (sum would be %.3f)", rms, 1.6 / std::sqrt(2.0)));
	}

	{
		FrameQueue a;
		FrameQueue b;
		AudioMixer mixer(48000);
		mixer.addSource(&a);
		mixer.addSource(&b);

		a.append(std::vector<float>(1000 * 2, 0.1F));
		b.append(std::vector<float>(600 * 2, 0.1F));
		auto part = mixAll(mixer, false);
		check(mixer.mixed() == 600, "mixing waits for the slower source", fmt("mixed %.0f", static_cast<double>(mixer.mixed())));

		b.append(std::vector<float>(100 * 2, 0.1F));
		auto more = mixAll(mixer, false);
		auto rest = mixAll(mixer, true);
		auto total = (part.size() + more.size() + rest.size()) / 2;
		auto tailOk = true;
		for (auto n = rest.size() / 2 - 300; n < rest.size() / 2; ++n) tailOk = tailOk && std::abs(rest[n * 2] - std::lrint(0.1 * 32767)) <= 1;
		check(total == 1000 && tailOk, "stopping mixes what the faster source has left", fmt("frames %.0f", static_cast<double>(total)));
	}

	{
		FrameQueue a;
		AudioMixer mixer(48000);
		mixer.addSource(&a);
		std::vector<float> click(4800 * 2, 0.0F);
		click[2400 * 2] = 3.0F;
		click[2400 * 2 + 1] = -2.0F;
		a.append(click);
		auto out = mixAll(mixer, true);
		auto peakAt = std::size_t {0};
		auto peak = 0;
		for (std::size_t n = 0; n < out.size() / 2; ++n) {
			if (std::abs(out[n * 2]) > peak) {
				peak = std::abs(out[n * 2]);
				peakAt = n;
			}
		}
		check(peakAt == 2400 && peak <= static_cast<int>(PeakLimiter::THRESHOLD * 32767) + 1,
		      "a 3x overload spike comes out at its own position under the ceiling",
		      fmt("at %.0f, peak %.0f", static_cast<double>(peakAt), peak));
	}
}

void testPipeline() {
	Device system {48000, 90, 480, 0.2, 0.02, 0.01, 440, 0.6};
	Device mic {16000, -700, 160, 0.8, 0.15, 0.02, 1000, 0.6, true};
	auto systemSim = simulate(system, 48000, 60.0, 0.1);
	auto micSim = simulate(mic, 48000, 60.0, 0.2);

	FrameQueue a;
	FrameQueue b;
	a.append(systemSim.out);
	b.append(micSim.out);
	AudioMixer mixer(48000);
	mixer.addSource(&a);
	mixer.addSource(&b);
	std::vector<std::int16_t> out;
	mixer.mix(true, out);

	std::vector<float> asFloat(out.size());
	for (std::size_t i = 0; i < out.size(); ++i) asFloat[i] = out[i] / 32767.0F;
	auto frames = asFloat.size() / 2;
	auto s = sinePhase(asFloat, frames - 3 * 48000, 48000, 440, 48000);
	auto m = sinePhase(asFloat, frames - 3 * 48000, 48000, 1000, 48000);
	auto peak = 0;
	for (auto v: out) peak = std::max(peak, std::abs(static_cast<int>(v)));

	check(std::abs(s.offset) * 1000 < 0.5 && std::abs(m.offset) * 1000 < 0.5,
	      "system audio and microphone stay aligned with each other after a minute",
	      fmt("system %.3f ms, microphone %.3f ms", s.offset * 1000, m.offset * 1000));
	check(peak <= static_cast<int>(PeakLimiter::THRESHOLD * 32767) + 1, "the mix of both stays under the ceiling", fmt("peak %.0f", peak));
}

void testStop() {
	Device system {48000, 90, 480, 0.2};
	Device mic {16000, -700, 160, 0.8, 0.15, 0.02, 1000, 0.5, true};
	auto stopAt = 7.0;
	auto systemSim = simulate(system, 48000, stopAt, 0.1, [](double t) { return t < 4.0; });
	auto micSim = simulate(mic, 48000, stopAt, 0.2);

	std::vector<float> tail;
	systemSim.stream.padTo(static_cast<std::int64_t>(stopAt * 48000), tail);
	systemSim.out.insert(systemSim.out.end(), tail.begin(), tail.end());
	tail.clear();
	micSim.stream.padTo(static_cast<std::int64_t>(stopAt * 48000), tail);
	micSim.out.insert(micSim.out.end(), tail.begin(), tail.end());

	FrameQueue a;
	FrameQueue b;
	a.append(systemSim.out);
	b.append(micSim.out);
	AudioMixer mixer(48000);
	mixer.addSource(&a);
	mixer.addSource(&b);
	std::vector<std::int16_t> out;
	mixer.mix(true, out);
	auto frames = static_cast<double>(out.size() / 2);
	check(std::abs(frames - stopAt * 48000) <= 1.0,
	      "after stopping, both sources end at the stop time",
	      fmt("frames %.0f, want %.0f (system %.0f, microphone %.0f)", frames, stopAt * 48000, static_cast<double>(systemSim.stream.written()), static_cast<double>(micSim.stream.written())));

	AlignedStream unstamped;
	unstamped.configure(48000, 48000);
	std::vector<float> packet(480 * 2, 0.25F);
	std::vector<float> sink;
	for (auto i = 0; i < 1000; ++i) {
		unstamped.push(packet.data(), 480, std::nullopt, sink);
		unstamped.padTo(static_cast<std::int64_t>((i * 0.01 - 0.2) * 48000), sink);
	}
	check(std::abs(static_cast<double>(unstamped.written()) - 480000.0) < 100,
	      "a device that never timestamps its packets still produces a continuous stream",
	      fmt("written %.0f", static_cast<double>(unstamped.written())));
}

void benchmark() {
	AlignedStream stream;
	stream.configure(44100, 48000);
	std::vector<float> packet(441 * 2);
	for (std::size_t n = 0; n < 441; ++n) packet[n * 2] = packet[n * 2 + 1] = static_cast<float>(std::sin(n * 0.05));
	std::vector<float> out;
	FrameQueue q;
	AudioMixer mixer(48000);
	mixer.addSource(&q);
	std::vector<std::int16_t> pcm;

	auto begin = std::chrono::steady_clock::now();
	auto seconds = 600;
	for (auto i = 0; i < seconds * 100; ++i) {
		out.clear();
		stream.push(packet.data(), 441, i * 480.0, out);
		q.append(out);
		if (i % 3 == 0) mixer.mix(false, pcm);
	}
	auto elapsed = std::chrono::duration<double>(std::chrono::steady_clock::now() - begin).count();
	std::printf("INFO  %d s of 44.1 -> 48 kHz stereo through resampler, queue, mixer and limiter in %.3f s (%.3f%% of one core)\n", seconds, elapsed, elapsed / seconds * 100);
}

} // namespace

int main() {
	testWeights();
	testConvert();
	testResampler();
	testDrift();
	testGaps();
	testOverlap();
	testMixer();
	testPipeline();
	testStop();
	benchmark();

	std::printf("%s (%d failed)\n", failures == 0 ? "ALL PASSED" : "FAILURES", failures);
	return failures == 0 ? 0 : 1;
}
