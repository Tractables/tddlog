cmake_minimum_required(VERSION 3.16)
# The version is the crate's, which tddlog_version() reports.
file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml" TDDLOG_VERSION REGEX "^version = " LIMIT_COUNT 1)
string(REGEX REPLACE "^version = \"([^\"]+)\"$" "\\1" TDDLOG_VERSION "${TDDLOG_VERSION}")
project(tddlog_c VERSION ${TDDLOG_VERSION} LANGUAGES C)
include(GNUInstallDirs)

# The library is built by cargo, into the build directory. The number of
# jobs is cargo's to choose, or CARGO_BUILD_JOBS's.
find_program(CARGO cargo REQUIRED)
set(RUST_LIBRARY
  "${CMAKE_CURRENT_BINARY_DIR}/cargo/release/${CMAKE_SHARED_LIBRARY_PREFIX}tddlog_c${CMAKE_SHARED_LIBRARY_SUFFIX}")
add_custom_target(tddlog_native ALL
  COMMAND "${CARGO}" build --release
          --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
          --target-dir "${CMAKE_CURRENT_BINARY_DIR}/cargo"
  BYPRODUCTS "${RUST_LIBRARY}" VERBATIM)
add_library(tddlog::tddlog SHARED IMPORTED GLOBAL)
set_target_properties(tddlog::tddlog PROPERTIES IMPORTED_LOCATION "${RUST_LIBRARY}"
  INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/include")
if(APPLE)
  set_target_properties(tddlog::tddlog PROPERTIES IMPORTED_SONAME "@rpath/libtddlog_c.dylib")
else()
  set_target_properties(tddlog::tddlog PROPERTIES IMPORTED_NO_SONAME TRUE)
endif()
add_dependencies(tddlog::tddlog tddlog_native)

option(TDDLOG_BUILD_TESTS "Build and register the C tests and the example" ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/bin")
foreach(config DEBUG RELEASE RELWITHDEBINFO MINSIZEREL)
  set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_${config} "${CMAKE_CURRENT_BINARY_DIR}/bin")
endforeach()
function(tddlog_consumer name source)
  add_executable(${name} "${source}")
  target_link_libraries(${name} PRIVATE tddlog::tddlog)
  set_target_properties(${name} PROPERTIES C_STANDARD 11 C_STANDARD_REQUIRED ON)
  target_compile_options(${name} PRIVATE -Wall -Wextra -Werror)
  target_link_libraries(${name} PRIVATE m)
endfunction()
if(TDDLOG_BUILD_TESTS)
  enable_testing()
  tddlog_consumer(alarm examples/alarm.c)
  add_test(NAME example_alarm COMMAND alarm)
  # A varied dozen of ProbLog's system tests, whose expected outcomes the
  # test reads from their comments.
  set(PROBLOG "${CMAKE_CURRENT_SOURCE_DIR}/../../tests/problog")
  set(PROGRAMS 00_trivial_and 00_trivial_not 00_trivial_or 10_cards 11_ads_numerical
      3_tossing_coin 4_bayesian_net 7_probabilistic_graph 8_smokers_network add negation
      negative_query smokers_or tc_3)
  list(TRANSFORM PROGRAMS PREPEND "${PROBLOG}/")
  list(TRANSFORM PROGRAMS APPEND ".pl")
  tddlog_consumer(answers tests/answers.c)
  add_test(NAME answers COMMAND answers ${PROGRAMS})
endif()

install(FILES include/tddlog.h DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
install(FILES "${RUST_LIBRARY}" DESTINATION "${CMAKE_INSTALL_LIBDIR}")
install(FILES ../../LICENSE ../../NOTICE ../../THIRD-PARTY.md
  DESTINATION "${CMAKE_INSTALL_DATADIR}/licenses/tddlog")

# The CMake package: find_package(tddlog CONFIG) and the target tddlog::tddlog.
include(CMakePackageConfigHelpers)
configure_package_config_file(tddlogConfig.cmake.in "${CMAKE_CURRENT_BINARY_DIR}/tddlogConfig.cmake"
  INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/tddlog"
  PATH_VARS CMAKE_INSTALL_INCLUDEDIR CMAKE_INSTALL_LIBDIR)
write_basic_package_version_file("${CMAKE_CURRENT_BINARY_DIR}/tddlogConfigVersion.cmake"
  VERSION "${PROJECT_VERSION}" COMPATIBILITY SameMinorVersion)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/tddlogConfig.cmake" "${CMAKE_CURRENT_BINARY_DIR}/tddlogConfigVersion.cmake"
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/tddlog")

# The pkg-config file, which finds the installation relative to itself, so a
# prefix can be moved.
file(RELATIVE_PATH TDDLOG_PC_PREFIX "/prefix/${CMAKE_INSTALL_LIBDIR}/pkgconfig" "/prefix")
configure_file(tddlog.pc.in "${CMAKE_CURRENT_BINARY_DIR}/tddlog.pc" @ONLY)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/tddlog.pc" DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig")
