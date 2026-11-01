const { readFileSync } = require('fs')
let nativeBinding = null
let __napiLoadedBindingTarget = 'native'
let __napiRequiredNativeBinding = null
const loadErrors = []

const isMusl = () => {
  let musl = false
  if (process.platform === 'linux') {
    musl = isMuslFromFilesystem()
    if (musl === null) {
      musl = isMuslFromReport()
    }
    if (musl === null) {
      musl = isMuslFromChildProcess()
    }
  }
  return musl
}

const isFileMusl = (f) => f.includes('libc.musl-') || f.includes('ld-musl-')

const isMuslFromFilesystem = () => {
  try {
    return readFileSync('/usr/bin/ldd', 'utf-8').includes('musl')
  } catch {
    return null
  }
}

const isMuslFromReport = () => {
  let report = null
  if (process.report && typeof process.report.getReport === 'function') {
    process.report.excludeNetwork = true
    report = process.report.getReport()
  }
  if (!report) {
    return null
  }
  if (report.header && report.header.glibcVersionRuntime) {
    return false
  }
  if (Array.isArray(report.sharedObjects)) {
    if (report.sharedObjects.some(isFileMusl)) {
      return true
    }
  }
  return false
}

const isMuslFromChildProcess = () => {
  try {
    return require('child_process').execSync('ldd --version', { encoding: 'utf8' }).includes('musl')
  } catch (e) {
    return false
  }
}

function requireNative() {
  if (process.env.NAPI_RS_NATIVE_LIBRARY_PATH) {
    try {
      const overrideBinding = require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH)
      __napiLoadedBindingTarget =
        overrideBinding && typeof overrideBinding.__napiBindingTarget === 'string'
          ? overrideBinding.__napiBindingTarget
          : 'native'
      return overrideBinding
    } catch (err) {
      loadErrors.push(err)
    }
  } else if (process.platform === 'android') {
    if (process.arch === 'arm64') {
      try {
        return require('./markers.android-arm64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-android-arm64')
        const bindingPackageVersion = require('@mdrs/markers-android-arm64/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else if (process.arch === 'arm') {
      try {
        return require('./markers.android-arm-eabi.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-android-arm-eabi')
        const bindingPackageVersion = require('@mdrs/markers-android-arm-eabi/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else {
      loadErrors.push(new Error(`Unsupported architecture on Android ${process.arch}`))
    }
  } else if (process.platform === 'win32') {
    if (process.arch === 'x64') {
      if ((process.config && process.config.variables && process.config.variables.shlib_suffix === 'dll.a') || (process.config && process.config.variables && process.config.variables.node_target_type === 'shared_library')) {
        try {
          return require('./markers.win32-x64-gnu.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-win32-x64-gnu')
          const bindingPackageVersion = require('@mdrs/markers-win32-x64-gnu/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      } else {
        try {
          return require('./markers.win32-x64-msvc.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-win32-x64-msvc')
          const bindingPackageVersion = require('@mdrs/markers-win32-x64-msvc/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      }
    } else if (process.arch === 'ia32') {
      try {
        return require('./markers.win32-ia32-msvc.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-win32-ia32-msvc')
        const bindingPackageVersion = require('@mdrs/markers-win32-ia32-msvc/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else if (process.arch === 'arm64') {
      try {
        return require('./markers.win32-arm64-msvc.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-win32-arm64-msvc')
        const bindingPackageVersion = require('@mdrs/markers-win32-arm64-msvc/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else {
      loadErrors.push(new Error(`Unsupported architecture on Windows: ${process.arch}`))
    }
  } else if (process.platform === 'darwin') {
    try {
      return require('./markers.darwin-universal.node')
    } catch (e) {
      loadErrors.push(e)
    }
    try {
      const binding = require('@mdrs/markers-darwin-universal')
      const bindingPackageVersion = require('@mdrs/markers-darwin-universal/package.json').version
      if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
        throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
      }
      return binding
    } catch (e) {
      loadErrors.push(e)
    }
    if (process.arch === 'x64') {
      try {
        return require('./markers.darwin-x64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-darwin-x64')
        const bindingPackageVersion = require('@mdrs/markers-darwin-x64/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else if (process.arch === 'arm64') {
      try {
        return require('./markers.darwin-arm64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-darwin-arm64')
        const bindingPackageVersion = require('@mdrs/markers-darwin-arm64/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else {
      loadErrors.push(new Error(`Unsupported architecture on macOS: ${process.arch}`))
    }
  } else if (process.platform === 'freebsd') {
    if (process.arch === 'x64') {
      try {
        return require('./markers.freebsd-x64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-freebsd-x64')
        const bindingPackageVersion = require('@mdrs/markers-freebsd-x64/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else if (process.arch === 'arm64') {
      try {
        return require('./markers.freebsd-arm64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-freebsd-arm64')
        const bindingPackageVersion = require('@mdrs/markers-freebsd-arm64/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else if (process.arch === 'ppc64') {
      try {
        return require('./markers.freebsd-ppc64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-freebsd-ppc64')
        const bindingPackageVersion = require('@mdrs/markers-freebsd-ppc64/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else {
      loadErrors.push(new Error(`Unsupported architecture on FreeBSD: ${process.arch}`))
    }
  } else if (process.platform === 'linux') {
    if (process.arch === 'x64') {
      if (isMusl()) {
        try {
          return require('./markers.linux-x64-musl.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-x64-musl')
          const bindingPackageVersion = require('@mdrs/markers-linux-x64-musl/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      } else {
        try {
          return require('./markers.linux-x64-gnu.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-x64-gnu')
          const bindingPackageVersion = require('@mdrs/markers-linux-x64-gnu/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      }
    } else if (process.arch === 'arm64') {
      if (isMusl()) {
        try {
          return require('./markers.linux-arm64-musl.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-arm64-musl')
          const bindingPackageVersion = require('@mdrs/markers-linux-arm64-musl/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      } else {
        try {
          return require('./markers.linux-arm64-gnu.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-arm64-gnu')
          const bindingPackageVersion = require('@mdrs/markers-linux-arm64-gnu/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      }
    } else if (process.arch === 'arm') {
      if (isMusl()) {
        try {
          return require('./markers.linux-arm-musleabihf.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-arm-musleabihf')
          const bindingPackageVersion = require('@mdrs/markers-linux-arm-musleabihf/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      } else {
        try {
          return require('./markers.linux-arm-gnueabihf.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-arm-gnueabihf')
          const bindingPackageVersion = require('@mdrs/markers-linux-arm-gnueabihf/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      }
    } else if (process.arch === 'loong64') {
      if (isMusl()) {
        try {
          return require('./markers.linux-loong64-musl.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-loong64-musl')
          const bindingPackageVersion = require('@mdrs/markers-linux-loong64-musl/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      } else {
        try {
          return require('./markers.linux-loong64-gnu.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-loong64-gnu')
          const bindingPackageVersion = require('@mdrs/markers-linux-loong64-gnu/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      }
    } else if (process.arch === 'riscv64') {
      if (isMusl()) {
        try {
          return require('./markers.linux-riscv64-musl.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-riscv64-musl')
          const bindingPackageVersion = require('@mdrs/markers-linux-riscv64-musl/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      } else {
        try {
          return require('./markers.linux-riscv64-gnu.node')
        } catch (e) {
          loadErrors.push(e)
        }
        try {
          const binding = require('@mdrs/markers-linux-riscv64-gnu')
          const bindingPackageVersion = require('@mdrs/markers-linux-riscv64-gnu/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
            throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
          return binding
        } catch (e) {
          loadErrors.push(e)
        }
      }
    } else if (process.arch === 'ppc64') {
      try {
        return require('./markers.linux-ppc64-gnu.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-linux-ppc64-gnu')
        const bindingPackageVersion = require('@mdrs/markers-linux-ppc64-gnu/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else if (process.arch === 's390x') {
      try {
        return require('./markers.linux-s390x-gnu.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-linux-s390x-gnu')
        const bindingPackageVersion = require('@mdrs/markers-linux-s390x-gnu/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else {
      loadErrors.push(new Error(`Unsupported architecture on Linux: ${process.arch}`))
    }
  } else if (process.platform === 'openharmony') {
    if (process.arch === 'arm64') {
      try {
        return require('./markers.openharmony-arm64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-openharmony-arm64')
        const bindingPackageVersion = require('@mdrs/markers-openharmony-arm64/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else if (process.arch === 'x64') {
      try {
        return require('./markers.openharmony-x64.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-openharmony-x64')
        const bindingPackageVersion = require('@mdrs/markers-openharmony-x64/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else if (process.arch === 'arm') {
      try {
        return require('./markers.openharmony-arm.node')
      } catch (e) {
        loadErrors.push(e)
      }
      try {
        const binding = require('@mdrs/markers-openharmony-arm')
        const bindingPackageVersion = require('@mdrs/markers-openharmony-arm/package.json').version
        if (bindingPackageVersion !== '0.1.0-alpha' && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          throw new Error(`Native binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
        }
        return binding
      } catch (e) {
        loadErrors.push(e)
      }
    } else {
      loadErrors.push(new Error(`Unsupported architecture on OpenHarmony: ${process.arch}`))
    }
  } else {
    loadErrors.push(new Error(`Unsupported OS: ${process.platform}, architecture: ${process.arch}`))
  }
}

function createLoadErrorChain(errors) {
  return errors.reduce((previous, current) => {
    let message
    try {
      message =
        current && typeof current.message === 'string'
          ? current.message
          : String(current)
    } catch {
      message = 'Unknown error'
    }
    const error = new Error(message)
    error.cause = previous
    return error
  }, null)
}

const __napiWasiFlavors = ['wasm32-wasi']
const __napiWasiFlavor = process.env.NAPI_RS_WASI_FLAVOR
const __napiWasiFlavorRequested =
  typeof __napiWasiFlavor === 'string' && __napiWasiFlavor.length > 0
if (
  __napiWasiFlavorRequested &&
  __napiWasiFlavors.indexOf(__napiWasiFlavor) === -1
) {
  throw new Error(
    'Unsupported WASI flavor "' +
      __napiWasiFlavor +
      '". Available flavors: ' +
      __napiWasiFlavors.join(', '),
  )
}
const forceWasiError = process.env.NAPI_RS_FORCE_WASI === 'error'
const forceWasi =
  process.env.NAPI_RS_FORCE_WASI === 'true' ||
  forceWasiError ||
  __napiWasiFlavorRequested

if (!forceWasi) {
  nativeBinding = requireNative()
  __napiRequiredNativeBinding = nativeBinding
}

if (!nativeBinding || forceWasi) {
  let wasiBinding = null
  let wasiBindingLoaded = false
  const wasiBindingErrors = []
  const __napiWasiResolveCandidate = (specifier, isPackage, localArtifacts) => {
    try {
      require.resolve(specifier)
    } catch (resolveError) {
      if (!resolveError || resolveError.code !== 'MODULE_NOT_FOUND') {
        throw resolveError
      }
      if (isPackage) {
        try {
          require.resolve(specifier + '/package.json')
        } catch (packageError) {
          if (packageError && packageError.code === 'MODULE_NOT_FOUND') {
            return resolveError
          }
          throw resolveError
        }
        throw resolveError
      }
      return resolveError
    }
    if (localArtifacts) {
      let artifactError = null
      for (let i = 0; i < localArtifacts.length; i++) {
        try {
          require.resolve(localArtifacts[i])
          return null
        } catch (resolveError) {
          if (!resolveError || resolveError.code !== 'MODULE_NOT_FOUND') {
            throw resolveError
          }
          artifactError = resolveError
        }
      }
      return artifactError
    }
    return null
  }
  if (!wasiBindingLoaded && (!__napiWasiFlavorRequested || __napiWasiFlavor === 'wasm32-wasi')) {
    let candidateError = null
    let candidateFailed = false
    try {
      candidateError = __napiWasiResolveCandidate('./markers.wasi.cjs', false, ['./markers.wasm32-wasi.debug.wasm', './markers.wasm32-wasi.wasm'])
      candidateFailed = candidateError !== null
      if (!candidateFailed) {
        wasiBinding = require('./markers.wasi.cjs')
        nativeBinding = wasiBinding
        __napiLoadedBindingTarget = 'wasm32-wasi'
        wasiBindingLoaded = true
      }
    } catch (err) {
      candidateError = err
      candidateFailed = true
    }
    if (candidateFailed) {
      wasiBindingErrors.push(candidateError)
      loadErrors.push(candidateError)
    }
  }
  if (!wasiBindingLoaded && (!__napiWasiFlavorRequested || __napiWasiFlavor === 'wasm32-wasi')) {
    let candidateError = null
    let candidateFailed = false
    try {
      candidateError = __napiWasiResolveCandidate('@mdrs/markers-wasm32-wasi', true, undefined)
      candidateFailed = candidateError !== null
      if (!candidateFailed) {
        if (process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== '0') {
          const bindingPackageVersion = require('@mdrs/markers-wasm32-wasi/package.json').version
          if (bindingPackageVersion !== '0.1.0-alpha') {
            throw new Error(`WASI binding package version mismatch, expected 0.1.0-alpha but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`)
          }
        }
        wasiBinding = require('@mdrs/markers-wasm32-wasi')
        nativeBinding = wasiBinding
        __napiLoadedBindingTarget = 'wasm32-wasi'
        wasiBindingLoaded = true
      }
    } catch (err) {
      candidateError = err
      candidateFailed = true
    }
    if (candidateFailed) {
      wasiBindingErrors.push(candidateError)
      loadErrors.push(candidateError)
    }
  }
  if (
    !wasiBindingLoaded &&
    forceWasi &&
    !forceWasiError &&
    !__napiWasiFlavorRequested
  ) {
    nativeBinding = requireNative()
    __napiRequiredNativeBinding = nativeBinding
  }
  if ((forceWasiError || __napiWasiFlavorRequested) && !wasiBindingLoaded) {
    const error = new Error(
      __napiWasiFlavorRequested
        ? 'WASI binding for flavor "' + __napiWasiFlavor + '" not found'
        : 'WASI binding not found and NAPI_RS_FORCE_WASI is set to error',
    )
    error.cause = createLoadErrorChain(wasiBindingErrors)
    throw error
  }
}

if (!nativeBinding) {
  if (loadErrors.length > 0) {
    const error = new Error(
      `Cannot find native binding. ` +
        `npm has a bug related to optional dependencies (https://github.com/npm/cli/issues/4828). ` +
        'Please try `npm i` again after removing both package-lock.json and node_modules directory.',
    )
    error.cause = createLoadErrorChain(loadErrors)
    throw error
  }
  throw new Error(`Failed to load native binding`)
}

if (
  __napiLoadedBindingTarget === 'native' &&
  nativeBinding !== __napiRequiredNativeBinding
) {
  try {
    const __napiBindingTargetDescriptor = Object.getOwnPropertyDescriptor(
      nativeBinding,
      '__napiBindingTarget',
    )
    if (
      __napiBindingTargetDescriptor &&
      __napiWasiFlavors.indexOf(__napiBindingTargetDescriptor.value) !== -1 &&
      nativeBinding.__napiBindingTarget === __napiBindingTargetDescriptor.value
    ) {
      __napiLoadedBindingTarget = __napiBindingTargetDescriptor.value
    }
  } catch {
  }
}

function __napiStampBindingTarget(exportsObject, target) {
  if (
    Object.prototype.hasOwnProperty.call(exportsObject, '__napiBindingTarget')
  ) {
    if (exportsObject.__napiBindingTarget === target) {
      return target
    }
    const error = new Error(
      '`__napiBindingTarget` is reserved by the generated binding loader, but the loaded binding already exports it. Rename the export, e.g. #[napi(js_name = "...")].',
    )
    error.code = 'ERR_NAPI_BINDING_TARGET_CONFLICT'
    throw error
  }
  if (!Object.isExtensible(exportsObject)) {
    return target
  }
  try {
    Object.defineProperty(exportsObject, '__napiBindingTarget', {
      configurable: true,
      enumerable: true,
      value: target,
      writable: true,
    })
  } catch {
  }
  return target
}
module.exports.__napiBindingTarget = __napiStampBindingTarget(nativeBinding, __napiLoadedBindingTarget)
module.exports = nativeBinding
module.exports.convert = nativeBinding.convert
module.exports.convertAsync = nativeBinding.convertAsync
