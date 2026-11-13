try {
  require("react-native-get-random-values");
} catch (_) {}
try {
  const qc = require("react-native-quick-crypto");
  if (qc && qc.install) qc.install();
  else if (qc && qc.default && qc.default.install) qc.default.install();
} catch (_) {}

import { registerRootComponent } from "expo";
import App from "./src/App";
registerRootComponent(App);
