// Windows headless CLI for MicroTeX, mirroring the flags of gtkmm_main.cpp's `-headless` mode
// (outputdir/samples/prefix/textsize/foreground/background/input/output/padding/maxwidth) but
// rendering through the Qt backend (graphic_qt) into an SVG via QSvgGenerator instead of Cairo,
// so Windows doesn't need gtkmm/cairomm. Added for the ii-windows port; not part of upstream
// MicroTeX. MIT-licensed, same as the rest of this project.
//
// Usage matches what ii/services/LatexRenderer.qml spawns on Linux:
//   LaTeX.exe -headless -input=<tex> -output=<file.svg> -textsize=<n> -padding=<n>
//             -foreground=<color> [-background=<color>] -maxwidth=<n>

#include "latex.h"
#include "atom/atom_basic.h"
#include "graphic/graphic_basic.h"
#include "platform/qt/graphic_qt.h"
#include "utils/utf.h"
#include "utils/utils.h"

#include <QFile>
#include <QGuiApplication>
#include <QPainter>
#include <QSvgGenerator>

#include <cstdio>
#include <string>
#include <vector>

using namespace tex;
using namespace std;

namespace {

struct Options {
	string input;
	string output;
	float textSize = 20.f;
	color foreground = BLACK;
	color background = TRANSPARENT;
	float padding = 10.f;
	float maxWidth = 720.f;
};

bool startsWith(const string& s, const string& prefix) {
	return s.rfind(prefix, 0) == 0;
}

string valueOf(const string& opt) {
	auto pos = opt.find('=');
	return pos == string::npos ? string() : opt.substr(pos + 1);
}

Options parseOptions(const vector<string>& args) {
	Options o;
	for (const auto& a : args) {
		if (startsWith(a, "-textsize")) o.textSize = stof(valueOf(a));
		else if (startsWith(a, "-foreground")) o.foreground = ColorAtom::getColor(valueOf(a));
		else if (startsWith(a, "-background")) o.background = ColorAtom::getColor(valueOf(a));
		else if (startsWith(a, "-input")) o.input = valueOf(a);
		else if (startsWith(a, "-output")) o.output = valueOf(a);
		else if (startsWith(a, "-padding")) o.padding = stof(valueOf(a));
		else if (startsWith(a, "-maxwidth")) o.maxWidth = stof(valueOf(a));
	}
	if (o.textSize <= 0.f) o.textSize = 20.f;
	if (o.maxWidth <= 0.f) o.maxWidth = 720.f;
	return o;
}

int renderToSvg(const Options& o) {
	if (o.output.empty()) {
		fprintf(stderr, "Error: -output must be specified\n");
		return 1;
	}
	if (o.input.empty()) {
		fprintf(stderr, "Error: -input must be specified\n");
		return 1;
	}

	wstring code = utf82wide(o.input);
	auto* r = LaTeX::parse(code, (int) o.maxWidth, o.textSize, o.textSize / 3.f, o.foreground);
	const float w = r->getWidth() + o.padding * 2;
	const float h = r->getHeight() + o.padding * 2;

	QSvgGenerator gen;
	gen.setFileName(QString::fromStdString(o.output));
	gen.setSize(QSize((int) w, (int) h));
	gen.setViewBox(QRect(0, 0, (int) w, (int) h));
	gen.setTitle("MicroTeX");

	{
		QPainter painter(&gen);
		painter.setRenderHint(QPainter::Antialiasing, true);
		if (!isTransparent(o.background)) {
			painter.fillRect(
				QRectF(0, 0, w, h),
				QColor(color_r(o.background), color_g(o.background), color_b(o.background), color_a(o.background)));
		}
		Graphics2D_qt g2(&painter);
		r->draw(g2, o.padding, o.padding);
	}
	delete r;
	return 0;
}

}  // namespace

int main(int argc, char** argv) {
	// QFontDatabase (used by graphic_qt's Font_qt) needs a QGuiApplication for the platform
	// font plugin to be loaded, even though we never show a window.
	QGuiApplication app(argc, argv);

	vector<string> args(argv + 1, argv + argc);
	bool headless = false;
	for (const auto& a : args) {
		if (a == "-headless") headless = true;
	}
	if (!headless) {
		fprintf(stderr, "LaTeX.exe (ii-windows headless build) only supports -headless mode.\n");
		return 1;
	}

	LaTeX::init("res");
	Options o = parseOptions(args);
	int rc = renderToSvg(o);
	LaTeX::release();
	return rc;
}
