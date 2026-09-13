#pragma once

#include <QObject>
#include <QQmlEngine>

// Set true while the custom titlebar is being dragged (see
// TitleBar.qml) so other continuously-animating things - currently
// AnimatedBackground's aurora/particles/grid - can pause themselves for
// the duration. Moving the window is already competing with the render
// thread for every frame; not also asking it to composite a
// perpetually-animating background on top measurably reduces the
// glitching that shows up on longer drags.
class WindowDragState : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(bool active READ active WRITE setActive NOTIFY activeChanged)

public:
    static WindowDragState *create(QQmlEngine *, QJSEngine *);

    explicit WindowDragState(QObject *parent = nullptr);

    bool active() const { return m_active; }
    void setActive(bool active);

signals:
    void activeChanged();

private:
    bool m_active = false;
};
