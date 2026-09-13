#include "AdapterFactory.h"
#include "YotsubaAdapter.h"
#include "LynxchanAdapter.h"

ChanAdapter *AdapterFactory::forSite(const QVariantMap &site)
{
    static YotsubaAdapter yotsuba;
    static LynxchanAdapter lynxchan;

    if (site.value("schema").toString() == QStringLiteral("lynxchan"))
        return &lynxchan;
    return &yotsuba;
}
