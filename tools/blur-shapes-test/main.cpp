#include <cmath>
#include <cstdio>
#include <optional>

#include <qguiapplication.h>
#include <qqmlcomponent.h>
#include <qqmlengine.h>
#include <qquickitem.h>
#include <qquickwindow.h>
#include <qregion.h>
#include <qstring.h>

#include "blur_shapes.hpp"

using qs::windows::BlurShape;
using qs::windows::BlurShapeQuery;
using qs::windows::collectBlurShapes;

namespace {

int failures = 0; // NOLINT

struct Expect {
	QRectF rect;
	qreal radius;
	QRectF clip;
};

QString describe(const BlurShape& shape) {
	return QString("%1,%2 %3x%4 r%5 clip %6,%7 %8x%9")
	    .arg(shape.rect.x())
	    .arg(shape.rect.y())
	    .arg(shape.rect.width())
	    .arg(shape.rect.height())
	    .arg(shape.radius)
	    .arg(shape.clip.x())
	    .arg(shape.clip.y())
	    .arg(shape.clip.width())
	    .arg(shape.clip.height());
}

bool close(qreal a, qreal b) { return std::abs(a - b) < 0.01; }
bool closeRect(const QRectF& a, const QRectF& b) {
	return close(a.x(), b.x()) && close(a.y(), b.y()) && close(a.width(), b.width())
	    && close(a.height(), b.height());
}

void check(
    const char* name,
    QQmlEngine& engine,
    const QByteArray& qml,
    BlurShapeQuery query,
    const QList<Expect>& expected,
    QSize windowSize = QSize(400, 300),
    QColor windowColor = Qt::transparent
) {
	QQuickWindow window;
	window.resize(windowSize);
	window.setColor(windowColor);

	QQmlComponent component(&engine);
	component.setData("import QtQuick\n" + qml, QUrl("file:///test.qml"));
	auto* object = component.create();
	auto* item = qobject_cast<QQuickItem*>(object);

	if (item == nullptr) {
		std::printf("FAIL %s: QML error %s\n", name, qPrintable(component.errorString()));
		failures++;
		delete object;
		return;
	}

	item->setParentItem(window.contentItem());

	auto result = collectBlurShapes(&window, query);
	auto ok = result.shapes.size() == expected.size();

	for (qsizetype i = 0; ok && i < expected.size(); i++) {
		const auto& got = result.shapes.at(i);
		const auto& want = expected.at(i);
		ok = closeRect(got.rect, want.rect) && close(got.radius, want.radius)
		  && (want.clip.isNull() || closeRect(got.clip, want.clip));
	}

	std::printf("%s %s\n", ok ? "ok  " : "FAIL", name);

	if (!ok) {
		failures++;
		for (const auto& shape: result.shapes) std::printf("     got  %s\n", qPrintable(describe(shape)));
		for (const auto& want: expected) {
			std::printf(
			    "     want %s\n",
			    qPrintable(describe(BlurShape {.rect = want.rect, .radius = want.radius, .clip = want.clip}))
			);
		}
	}

	delete item;
}

}

int main(int argc, char** argv) {
	QGuiApplication app(argc, argv);
	QQmlEngine engine;

	auto ii = BlurShapeQuery {.ignoreAlpha = 0.79};
	auto win = QRectF(0, 0, 400, 300);

	check(
	    "sidebar background",
	    engine,
	    R"(Item { anchors.fill: parent
	        Item { x: 5; y: 5; width: 390; height: 290 } // shadow stand-in
	        Rectangle { x: 10; y: 10; width: 380; height: 280; radius: 20
	            color: Qt.rgba(0.1, 0.1, 0.1, 0.85); border.width: 1; border.color: "gray"
	            Rectangle { x: 10; y: 10; width: 100; height: 40; radius: 8; color: Qt.rgba(1, 1, 1, 0.5) }
	        }
	    })",
	    ii,
	    {{QRectF(11, 11, 378, 278), 19, win}}
	);

	check(
	    "opaque background needs no blur",
	    engine,
	    R"(Rectangle { x: 10; y: 10; width: 380; height: 280; radius: 20; color: "#202020"
	        Rectangle { x: 10; y: 10; width: 100; height: 40; color: Qt.rgba(1, 1, 1, 0.9) }
	    })",
	    ii,
	    {}
	);

	check(
	    "below ignoreAlpha: nothing (like Hyprland at alpha 0.78)",
	    engine,
	    R"(Rectangle { width: 400; height: 40; color: Qt.rgba(0, 0, 0, 0.78) })",
	    ii,
	    {}
	);

	check(
	    "transparent bar, translucent groups",
	    engine,
	    R"(Item { width: 400; height: 40
	        Rectangle { anchors.fill: parent; color: "transparent"
	            Row { x: 4; y: 4; spacing: 10
	                Rectangle { width: 100; height: 32; radius: 16; color: Qt.rgba(0.2, 0.2, 0.2, 0.9) }
	                Rectangle { width: 60; height: 32; radius: 16; color: Qt.rgba(0.2, 0.2, 0.2, 0.9) }
	                Rectangle { width: 60; height: 32; radius: 16; color: Qt.rgba(0.2, 0.2, 0.2, 0.3) }
	            }
	        }
	    })",
	    ii,
	    {{QRectF(5, 5, 98, 30), 15, win}, {QRectF(115, 5, 58, 30), 15, win}}
	);

	check(
	    "coverage of nested rectangles",
	    engine,
	    R"(Rectangle { width: 400; height: 100; color: Qt.rgba(0, 0, 0, 0.6)
	        Rectangle { x: 20; y: 20; width: 50; height: 50; radius: 4; color: Qt.rgba(0, 0, 0, 0.6) }
	    })",
	    ii,
	    {{QRectF(21, 21, 48, 48), 3, win}}
	);

	check(
	    "opacity multiplies",
	    engine,
	    R"(Item { opacity: 0.5
	        Rectangle { width: 100; height: 100; color: "black" }
	    })",
	    ii,
	    {}
	);

	check(
	    "opacity multiplies, ignoreAlpha 0",
	    engine,
	    R"(Item { opacity: 0.5
	        Rectangle { width: 100; height: 100; color: "black" }
	    })",
	    BlurShapeQuery {.ignoreAlpha = 0.0},
	    {{QRectF(1, 1, 98, 98), 0, win}}
	);

	check(
	    "invisible and transparent items are skipped",
	    engine,
	    R"(Item {
	        Rectangle { visible: false; width: 100; height: 100; color: Qt.rgba(0, 0, 0, 0.9) }
	        Rectangle { opacity: 0; width: 100; height: 100; color: Qt.rgba(0, 0, 0, 0.9) }
	    })",
	    ii,
	    {}
	);

	check(
	    "ancestor clip",
	    engine,
	    R"(Item { x: 50; y: 50; width: 200; height: 100; clip: true
	        Rectangle { y: 60; width: 200; height: 80; radius: 10; color: Qt.rgba(0, 0, 0, 0.9) }
	    })",
	    ii,
	    {{QRectF(51, 111, 198, 78), 9, QRectF(50, 50, 200, 100)}}
	);

	check(
	    "clipped out entirely",
	    engine,
	    R"(Item { width: 200; height: 100; clip: true
	        Rectangle { y: 150; width: 200; height: 80; color: Qt.rgba(0, 0, 0, 0.9) }
	    })",
	    ii,
	    {}
	);

	auto masked = ii;
	masked.hasMask = true;
	masked.mask = QRegion(0, 0, 200, 300);
	check(
	    "only shapes touching the mask",
	    engine,
	    R"(Item {
	        Rectangle { x: 10; width: 50; height: 50; color: Qt.rgba(0, 0, 0, 0.9) }
	        Rectangle { x: 300; width: 50; height: 50; color: Qt.rgba(0, 0, 0, 0.9) }
	    })",
	    masked,
	    {{QRectF(11, 1, 48, 48), 0, win}}
	);

	auto emptyMask = ii;
	emptyMask.hasMask = true;
	check(
	    "empty mask (click-through window) gets nothing",
	    engine,
	    R"(Rectangle { width: 50; height: 50; color: Qt.rgba(0, 0, 0, 0.9) })",
	    emptyMask,
	    {}
	);

	auto whole = BlurShapeQuery {};
	check(
	    "no ignoreAlpha: the whole window",
	    engine,
	    R"(Item {})",
	    whole,
	    {{QRectF(1, 1, 398, 298), 0, win}}
	);

	auto wholeMasked = whole;
	wholeMasked.hasMask = true;
	wholeMasked.mask = QRegion(10, 10, 100, 50);
	check(
	    "no ignoreAlpha: the mask",
	    engine,
	    R"(Item {})",
	    wholeMasked,
	    {{QRectF(11, 11, 98, 48), 0, win}}
	);

	check(
	    "translucent window color is the surface",
	    engine,
	    R"(Item {})",
	    ii,
	    {{QRectF(1, 1, 398, 298), 0, win}},
	    QSize(400, 300),
	    QColor::fromRgbF(0, 0, 0, 0.9f)
	);

	check(
	    "opaque window color: nothing",
	    engine,
	    R"(Rectangle { width: 50; height: 50; color: Qt.rgba(0, 0, 0, 0.9) })",
	    ii,
	    {},
	    QSize(400, 300),
	    Qt::white
	);

	auto hidpi = ii;
	hidpi.dpr = 1.5;
	check(
	    "dpr 1.5",
	    engine,
	    R"(Rectangle { x: 10; y: 20; width: 100; height: 40; radius: 12; color: Qt.rgba(0, 0, 0, 0.9) })",
	    hidpi,
	    {{QRectF(16, 31, 148, 58), 17, QRectF(0, 0, 600, 450)}}
	);

	check(
	    "scaled item scales the radius",
	    engine,
	    R"(Rectangle { x: 100; y: 100; width: 50; height: 50; radius: 10; scale: 2
	        color: Qt.rgba(0, 0, 0, 0.9) })",
	    ii,
	    {{QRectF(76, 76, 98, 98), 19, win}}
	);

	check(
	    "per corner radii take the roundest",
	    engine,
	    R"(Rectangle { width: 100; height: 60; radius: 4; topLeftRadius: 0; bottomRightRadius: 20
	        color: Qt.rgba(0, 0, 0, 0.9) })",
	    ii,
	    {{QRectF(1, 1, 98, 58), 19, win}}
	);

	check(
	    "radius clamps to half the size",
	    engine,
	    R"(Rectangle { width: 100; height: 20; radius: 50; color: Qt.rgba(0, 0, 0, 0.9) })",
	    ii,
	    {{QRectF(1, 1, 98, 18), 9, win}}
	);

	check(
	    "gradient counts with its most opaque stop",
	    engine,
	    R"(Rectangle { width: 100; height: 60
	        gradient: Gradient {
	            GradientStop { position: 0; color: Qt.rgba(0, 0, 0, 0.2) }
	            GradientStop { position: 1; color: Qt.rgba(0, 0, 0, 0.9) }
	        }
	    })",
	    ii,
	    {{QRectF(1, 1, 98, 58), 0, win}}
	);

	check(
	    "partly outside the window keeps its full rect",
	    engine,
	    R"(Rectangle { x: -50; y: 100; width: 100; height: 50; radius: 10; color: Qt.rgba(0, 0, 0, 0.9) })",
	    ii,
	    {{QRectF(-49, 101, 98, 48), 9, win}}
	);

	std::printf("%d failure(s)\n", failures);
	return failures;
}
