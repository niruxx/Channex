#pragma once

#include <QObject>
#include <QQmlEngine>
#include <QVariantList>
#include <QVariantMap>

class LightboxImageProvider;

// Mirrors src/store/useLightboxStore.ts + MediaLightbox.tsx's fetch
// behavior. Images are always pulled through NetworkClient into
// LightboxImageProvider rather than letting an Image element hit the
// remote URL directly (see that provider's doc comment for why); video
// files are played directly from their remote URL via QtMultimedia,
// which doesn't share the reliability problem the Tauri app worked
// around.
class LightboxController : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

    Q_PROPERTY(bool isOpen READ isOpen NOTIFY isOpenChanged)
    Q_PROPERTY(QVariantList files READ files NOTIFY filesChanged)
    Q_PROPERTY(int index READ index NOTIFY indexChanged)
    Q_PROPERTY(QVariantMap currentFile READ currentFile NOTIFY indexChanged)
    Q_PROPERTY(bool zoomed READ zoomed WRITE setZoomed NOTIFY zoomedChanged)
    Q_PROPERTY(QString loadState READ loadState NOTIFY loadStateChanged)
    Q_PROPERTY(QString imageSource READ imageSource NOTIFY loadStateChanged)

public:
    // Constructor stays private: a public one makes this look default-
    // constructible to QML_SINGLETON's factory detection, which can then
    // bypass create() and hand QML a disconnected instance.
    static LightboxController *create(QQmlEngine *, QJSEngine *);
    static LightboxController *instance();
    static void setImageProvider(LightboxImageProvider *provider);

    bool isOpen() const { return m_isOpen; }
    QVariantList files() const { return m_files; }
    int index() const { return m_index; }
    QVariantMap currentFile() const;
    bool zoomed() const { return m_zoomed; }
    void setZoomed(bool z);
    QString loadState() const { return m_loadState; }
    QString imageSource() const;

    Q_INVOKABLE void open(const QVariantList &files, int startIndex);
    Q_INVOKABLE void close();
    Q_INVOKABLE void next();
    Q_INVOKABLE void prev();
    Q_INVOKABLE void goTo(int index);

signals:
    void isOpenChanged();
    void filesChanged();
    void indexChanged();
    void zoomedChanged();
    void loadStateChanged();

private:
    explicit LightboxController(QObject *parent = nullptr);

    void loadCurrent();

    QVariantList m_files;
    int m_index = 0;
    bool m_isOpen = false;
    bool m_zoomed = false;
    QString m_loadState = QStringLiteral("loading");
    int m_cacheBust = 0;
    int m_requestSeq = 0;
};
