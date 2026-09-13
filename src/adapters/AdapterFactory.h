#pragma once

#include "ChanAdapter.h"

namespace AdapterFactory {

// Dispatches on site["schema"] ("lynxchan" vs anything else -> yotsuba),
// mirroring src/lib/adapters/index.ts::getAdapter(). Returned adapters
// are process-lifetime singletons (they hold no per-site state).
ChanAdapter *forSite(const QVariantMap &site);

}
