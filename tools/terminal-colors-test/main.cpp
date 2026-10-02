// Runs the Windows backend's terminal color harmonization (src/windows/system/
// terminal_colors_math.cpp, which is Qt-free) on cases read from stdin, one per line:
//   <dark|light> <harmony> <threshold> <fgBoost> <monochrome 0|1> <primaryKeyColor>
//   <surfaceContainerLow> <onSurface> <term0> ... <term15>
// (colors as #RRGGBB, numbers as Python float reprs or hex floats) and prints one line per case:
//   #RRGGBB x16 (term0..term15)
// compare.py feeds it the same cases as ii's generate_colors_material.py and diffs the output.
// With --hct-crc it prints, for each 65536-color block of sRGB (256 lines), the CRC-32 of the
// hue, chroma and tone doubles of Hct(argb) for every color in it, which compare.py --hct checks
// against materialyoucolor's Hct.from_int.

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <optional>
#include <sstream>
#include <string>

#include "cpp/cam/hct.h"
#include "terminal_colors_math.hpp"

namespace tc = qs::windows::sys::terminal_colors;

namespace {

// generate_colors_material.py's hex_to_argb: "#RRGGBB" only.
std::optional<tc::Argb> parseHex(const std::string& hex) {
	if (hex.size() != 7 || hex[0] != '#') return std::nullopt;
	char* end = nullptr;
	auto value = std::strtoul(hex.c_str() + 1, &end, 16);
	if (*end != '\0') return std::nullopt;
	return static_cast<tc::Argb>(value) | 0xFF000000u;
}

double parseNumber(const std::string& text) { return std::strtod(text.c_str(), nullptr); }

// zlib.crc32's CRC-32.
std::uint32_t crc32(std::uint32_t crc, const unsigned char* data, std::size_t size) {
	static std::uint32_t table[256] = {};
	if (table[1] == 0) {
		for (std::uint32_t i = 0; i < 256; i++) {
			std::uint32_t c = i;
			for (int k = 0; k < 8; k++) c = (c & 1) ? 0xEDB88320u ^ (c >> 1) : c >> 1;
			table[i] = c;
		}
	}
	crc = ~crc;
	for (std::size_t i = 0; i < size; i++) crc = table[(crc ^ data[i]) & 0xFF] ^ (crc >> 8);
	return ~crc;
}

int hctCrc() {
	for (std::uint32_t block = 0; block < 256; block++) {
		std::uint32_t crc = 0;
		for (std::uint32_t rgb = block << 16; rgb < (block + 1) << 16; rgb++) {
			auto hct = material_color_utilities::Hct(rgb | 0xFF000000u);
			double values[3] = {hct.get_hue(), hct.get_chroma(), hct.get_tone()};
			unsigned char bytes[sizeof(values)];
			std::memcpy(bytes, values, sizeof(values)); // little endian on every target here
			crc = crc32(crc, bytes, sizeof(bytes));
		}
		std::printf("%08x\n", crc);
	}
	return 0;
}

} // namespace

int main(int argc, char** argv) {
	if (argc > 1 && std::strcmp(argv[1], "--hct-crc") == 0) return hctCrc();

	std::string line;
	int lineNumber = 0;
	while (std::getline(std::cin, line)) {
		lineNumber++;
		if (line.empty() || line[0] == '#') continue;

		std::istringstream fields(line);
		std::string mode, harmony, threshold, fgBoost, monochrome, primary, scl, onSurface;
		fields >> mode >> harmony >> threshold >> fgBoost >> monochrome >> primary >> scl >> onSurface;

		auto inputs = tc::TerminalInputs {
		    .primaryKeyColor = parseHex(primary),
		    .surfaceContainerLow = parseHex(scl),
		    .onSurface = parseHex(onSurface),
		    .darkMode = mode == "dark",
		    .harmony = parseNumber(harmony),
		    .harmonizeThreshold = parseNumber(threshold),
		    .fgBoost = parseNumber(fgBoost),
		    .monochrome = monochrome == "1",
		};

		std::string out;
		for (int i = 0; i < 16; i++) {
			std::string baseHex;
			if (!(fields >> baseHex)) {
				std::fprintf(stderr, "line %d: expected 16 base colors\n", lineNumber);
				return 2;
			}

			auto name = "term" + std::to_string(i);
			auto color = tc::terminalColor(name, parseHex(baseHex), inputs);
			if (color) {
				char hex[8];
				std::snprintf(
				    hex,
				    sizeof(hex),
				    "#%02X%02X%02X",
				    (*color >> 16) & 0xFF,
				    (*color >> 8) & 0xFF,
				    *color & 0xFF
				);
				out += hex;
			} else {
				out += baseHex; // passed through, like the script's monochrome branch
			}
			out += i == 15 ? "\n" : " ";
		}

		std::fputs(out.c_str(), stdout);
	}

	return 0;
}
