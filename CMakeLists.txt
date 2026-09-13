cmake_minimum_required(VERSION 3.21)
project(channex_qt VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2 Network Multimedia)

# QML_FILES below span qml/, qml/components/, and qml/views/ - opt in to
# per-directory qmldir generation so each is importable, silencing the
# QTP0004 warning.
qt_policy(SET QTP0004 NEW)

qt_standard_project_setup(REQUIRES 6.5)

# `pragma Singleton` auto-detection isn't picking up qml/Theme.qml under the
# QTP0004 NEW per-directory qmldir split above (it registers as a plain type,
# so every Theme.xxx access in QML silently resolves to undefined at
# runtime). Force it explicitly rather than relying on scanning.
set_source_files_properties(qml/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)

qt_add_executable(channex_qt
    src/main.cpp
    src/settings/SettingsManager.h
    src/settings/SettingsManager.cpp
    src/network/NetworkClient.h
    src/network/NetworkClient.cpp
    src/adapters/ChanAdapter.h
    src/adapters/CommentFormatter.h
    src/adapters/CommentFormatter.cpp
    src/adapters/YotsubaAdapter.h
    src/adapters/YotsubaAdapter.cpp
    src/adapters/LynxchanAdapter.h
    src/adapters/LynxchanAdapter.cpp
    src/adapters/AdapterFactory.h
    src/adapters/AdapterFactory.cpp
    src/sites/SitesController.h
    src/sites/SitesController.cpp
    src/catalog/CatalogController.h
    src/catalog/CatalogController.cpp
    src/thread/ThreadController.h
    src/thread/ThreadController.cpp
    src/bookmarks/BookmarksController.h
    src/bookmarks/BookmarksController.cpp
    src/downloads/DownloadManager.h
    src/downloads/DownloadManager.cpp
    src/lightbox/LightboxController.h
    src/lightbox/LightboxController.cpp
    src/lightbox/LightboxImageProvider.h
    src/lightbox/LightboxImageProvider.cpp
    src/posting/LynxchanPoster.h
    src/posting/LynxchanPoster.cpp
    src/posting/ExternalReply.h
    src/posting/ExternalReply.cpp
    src/navigation/NavigationController.h
    src/navigation/NavigationController.cpp
    src/navigation/WindowDragState.h
    src/navigation/WindowDragState.cpp
    src/icons/IconProvider.h
    src/icons/IconProvider.cpp
)

qt_add_qml_module(channex_qt
    URI Channex
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/Theme.qml
        qml/EmptyState.qml
        qml/MediaLightbox.qml
        qml/components/TitleBar.qml
        qml/components/WindowControls.qml
        qml/components/Sidebar.qml
        qml/components/SiteSwitcher.qml
        qml/components/AddSiteDialog.qml
        qml/components/DownloadSelectionDialog.qml
        qml/components/BoardList.qml
        qml/components/AnimatedBackground.qml
        qml/components/ThreadCard.qml
        qml/components/ThreadListRow.qml
        qml/components/CatalogToolbar.qml
        qml/components/AppButton.qml
        qml/components/AppTextField.qml
        qml/components/AppTextArea.qml
        qml/components/AppComboBox.qml
        qml/components/AppCheckBox.qml
        qml/components/AppIcon.qml
        qml/views/CatalogView.qml
        qml/views/ThreadView.qml
        qml/views/PostCard.qml
        qml/views/ReplyComposer.qml
        qml/views/BookmarksView.qml
        qml/views/DownloadsView.qml
        qml/views/SettingsView.qml
        qml/views/OnboardingWizard.qml
    RESOURCES
        resources/icons/app.svg
        resources/icons/ui/grid.svg
        resources/icons/ui/rows.svg
        resources/icons/ui/list.svg
        resources/icons/ui/search.svg
        resources/icons/ui/refresh.svg
        resources/icons/ui/chevron-left.svg
        resources/icons/ui/chevron-right.svg
        resources/icons/ui/x.svg
        resources/icons/ui/minus.svg
        resources/icons/ui/square.svg
        resources/icons/ui/copy.svg
        resources/icons/ui/bookmark.svg
        resources/icons/ui/bookmark-fill.svg
        resources/icons/ui/download.svg
        resources/icons/ui/gear.svg
        resources/icons/ui/plus.svg
        resources/icons/ui/zoom-in.svg
        resources/icons/ui/play.svg
        resources/icons/ui/folder.svg
        resources/icons/ui/pause.svg
        resources/icons/ui/volume.svg
        resources/icons/ui/volume-mute.svg
        resources/icons/ui/check.svg
)

set_target_properties(channex_qt PROPERTIES
    WIN32_EXECUTABLE ON
    MACOSX_BUNDLE ON
)

# qt_add_qml_module's generated type-registration TU includes each
# QML_ELEMENT/QML_SINGLETON header as a bare <ClassName.h>, so every
# subdirectory holding one has to be on the include path directly -
# "src/" alone isn't enough since the headers live under src/<domain>/.
target_include_directories(channex_qt PRIVATE
    src
    src/adapters
    src/bookmarks
    src/catalog
    src/downloads
    src/icons
    src/lightbox
    src/navigation
    src/network
    src/posting
    src/settings
    src/sites
    src/thread
)

target_link_libraries(channex_qt PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::Network
    Qt6::Multimedia
)

include(GNUInstallDirs)
install(TARGETS channex_qt
    BUNDLE DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)
