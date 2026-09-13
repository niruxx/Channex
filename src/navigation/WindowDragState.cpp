#include "WindowDragState.h"

WindowDragState *WindowDragState::create(QQmlEngine *, QJSEngine *)
{
    return new WindowDragState();
}

WindowDragState::WindowDragState(QObject *parent)
    : QObject(parent)
{
}

void WindowDragState::setActive(bool active)
{
    if (m_active == active)
        return;
    m_active = active;
    emit activeChanged();
}
