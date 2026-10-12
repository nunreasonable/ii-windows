#include <chrono>
#include <cstdio>
#include <vector>

#include <qcolor.h>
#include <qcoreapplication.h>
#include <qdir.h>
#include <qfileinfo.h>
#include <qimage.h>
#include <qpainter.h>
#include <qpen.h>
#include <qrect.h>
#include <qstring.h>

#include "region_finder.hpp"

using qs::windows::image::detectContentRegions;
using qs::windows::image::RegionFinderOptions;

namespace {

void drawAndSave(const QImage& original, const std::vector<QRect>& regions, const QString& outPath) {
	auto canvas = original.convertToFormat(QImage::Format_RGB32);
	QPainter painter(&canvas);
	painter.setPen(QPen(QColor(255, 32, 32), 3));
	for (const auto& r : regions) painter.drawRect(r);
	painter.end();
	canvas.save(outPath);
}

} // namespace

int main(int argc, char** argv) {
	QCoreApplication app(argc, argv);

	if (argc < 3) {
		std::fprintf(stderr, "usage: region_finder_test <out-dir> <image...>\n");
		return 1;
	}

	QString outDir = argv[1];
	QDir().mkpath(outDir);

	for (int i = 2; i < argc; i++) {
		QString path = argv[i];
		QImage image(path);
		if (image.isNull()) {
			std::fprintf(stderr, "skip (could not load): %s\n", argv[i]);
			continue;
		}

		RegionFinderOptions options;
		options.minWidth = 200;
		options.minHeight = 100;
		options.maxWidth = static_cast<int>(image.width() * 0.5);
		options.maxHeight = static_cast<int>(image.height() * 0.5);

		auto t0 = std::chrono::steady_clock::now();
		auto regions = detectContentRegions(image, options);
		auto t1 = std::chrono::steady_clock::now();
		auto ms = std::chrono::duration<double, std::milli>(t1 - t0).count();

		auto base = QFileInfo(path).completeBaseName();
		std::printf(
		    "%s: %dx%d, %zu regions, %.2f ms\n",
		    qPrintable(base),
		    image.width(),
		    image.height(),
		    regions.size(),
		    ms
		);

		for (const auto& r : regions) {
			std::printf("  at=[%d,%d] size=[%d,%d]\n", r.x(), r.y(), r.width(), r.height());
		}

		drawAndSave(image, regions, outDir + "/" + base + "-regions.png");
	}

	return 0;
}
