# Signing release APK

```bash
keytool -genkey -v -keystore liberty.jks -keyalg RSA -keysize 2048 -validity 10000 -alias liberty
```

`android/key.properties`:
```
storePassword=...
keyPassword=...
keyAlias=liberty
storeFile=../liberty.jks
```

Wire into `app/build.gradle` signingConfigs.release. Never commit jks or key.properties.
