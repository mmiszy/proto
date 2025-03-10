#[macro_export]
macro_rules! generate_download_install_tests {
    ($id:literal, $version:literal) => {
        #[tokio::test]
        async fn downloads_verifies_installs_tool() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let plugin = create_plugin($id, sandbox.path());
            let bin_params = plugin.locate_bins(LocateBinsInput {
                env: plugin.tool.get_environment().unwrap(),
                ..LocateBinsInput::default()
            });

            let mut tool = plugin.tool;

            tool.setup($version).await.unwrap();

            // Check install dir exists
            let base_dir = sandbox.path().join(".proto/tools").join($id).join($version);

            assert_eq!(tool.get_install_dir().unwrap(), base_dir);
            assert!(base_dir.exists());

            // Check bin path exists
            assert_eq!(
                tool.get_bin_path().unwrap(),
                &base_dir.join(bin_params.bin_path.unwrap_or($id.into()))
            );

            // Check global bin exists
            assert!(sandbox
                .path()
                .join(".proto/bin")
                .join(if cfg!(windows) {
                    format!("{}.cmd", $id)
                } else {
                    $id.into()
                })
                .exists());
        }

        #[tokio::test]
        async fn downloads_prebuilt_and_checksum_to_temp() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let plugin = create_plugin($id, sandbox.path());
            let mut tool = plugin.tool;

            tool.version = Some(String::from($version));

            let download_file = tool.get_download_path().unwrap();
            let checksum_file = tool.get_checksum_path().unwrap();

            assert!(!download_file.exists());
            assert!(!checksum_file.exists());

            tool.download(&download_file, None).await.unwrap();

            assert!(download_file.exists());

            if tool.get_checksum_url().unwrap().is_some() {
                tool.download_checksum(&checksum_file, None).await.unwrap();

                assert!(checksum_file.exists());
            }
        }

        #[tokio::test]
        async fn doesnt_download_if_file_exists() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let plugin = create_plugin($id, sandbox.path());
            let mut tool = plugin.tool;

            tool.version = Some(String::from($version));

            let download_file = tool.get_download_path().unwrap();
            let checksum_file = tool.get_checksum_path().unwrap();

            assert!(tool.download(&download_file, None).await.unwrap());
            assert!(!tool.download(&download_file, None).await.unwrap());

            if tool.get_checksum_url().unwrap().is_some() {
                assert!(tool.download_checksum(&checksum_file, None).await.unwrap());
                assert!(!tool.download_checksum(&checksum_file, None).await.unwrap());
            }
        }

        #[tokio::test]
        #[should_panic(expected = "InstallMissingDownload")]
        async fn errors_for_missing_downloads_when_installing() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let plugin = create_plugin($id, sandbox.path());
            let mut tool = plugin.tool;

            tool.version = Some(String::from($version));

            let dir = tool.get_install_dir().unwrap();

            tool.install(&dir, &tool.get_download_path().unwrap())
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn doesnt_install_if_dir_exists() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let plugin = create_plugin($id, sandbox.path());
            let tool = plugin.tool;

            let dir = tool.get_install_dir().unwrap();

            std::fs::create_dir_all(&dir).unwrap();

            assert!(!tool
                .install(&dir, &tool.get_download_path().unwrap())
                .await
                .unwrap());
        }

        #[tokio::test]
        #[should_panic(expected = "VerifyInvalidChecksum")]
        async fn errors_for_checksum_mismatch() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let plugin = create_plugin($id, sandbox.path());
            let mut tool = plugin.tool;

            if tool.get_checksum_url().unwrap().is_none() {
                panic!("VerifyInvalidChecksum");
            }

            tool.version = Some(String::from($version));

            let download_file = tool.get_download_path().unwrap();
            let checksum_file = tool.get_checksum_path().unwrap();

            tool.download(&download_file, None).await.unwrap();
            tool.download_checksum(&checksum_file, None).await.unwrap();

            // Empty the checksum file
            std::fs::write(&checksum_file, "").unwrap();

            tool.verify_checksum(&download_file, &checksum_file)
                .await
                .unwrap();
        }
    };
}

#[macro_export]
macro_rules! generate_resolve_versions_tests {
    ($id:literal, { $( $k:literal => $v:literal, )* }) => {
        #[tokio::test]
        async fn updates_plugin_version() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let mut plugin = create_plugin($id, sandbox.path());

            assert_ne!(
                plugin.tool.resolve_version("latest").await.unwrap(),
                "latest"
            );
            assert_ne!(plugin.tool.get_resolved_version(), "latest");
        }

        #[tokio::test]
        async fn resolve_version_or_alias() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let mut plugin = create_plugin($id, sandbox.path());

            $(
                assert_eq!(
                    plugin.tool.resolve_version($k).await.unwrap(),
                    $v
                );
                plugin.tool.version = None;
            )*
        }

        #[tokio::test]
        async fn resolve_latest_alias() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let mut plugin = create_plugin($id, sandbox.path());

            assert_ne!(
                plugin.tool.resolve_version("latest").await.unwrap(),
                "latest"
            );
        }

        // #[tokio::test]
        // async fn resolve_custom_alias() {
        //     use proto_core::*;

        //     let sandbox = create_empty_sandbox();

        //     sandbox.create_file(
        //         format!(".proto/tools/{}/manifest.json", $id),
        //         r#"{"aliases":{"example":"1.0.0"}}"#,
        //     );

        //     let mut plugin = create_plugin($id, sandbox.path());

        //     assert_eq!(
        //         plugin.tool.resolve_version("example").await.unwrap(),
        //         "1.0.0"
        //     );
        // }

        #[tokio::test]
        #[should_panic(expected = "VersionUnknownAlias(\"unknown\")")]
        async fn errors_invalid_alias() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let mut plugin = create_plugin($id, sandbox.path());

            plugin.tool.resolve_version("unknown").await.unwrap();
        }

        #[tokio::test]
        #[should_panic(expected = "VersionResolveFailed(\"99.99.99\")")]
        async fn errors_invalid_version() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let mut plugin = create_plugin($id, sandbox.path());

            plugin.tool.resolve_version("99.99.99").await.unwrap();
        }
    };
}

#[macro_export]
macro_rules! generate_global_shims_test {
    ($id:literal) => {
        generate_global_shims_test!($id, []);
    };
    ($id:literal, [ $($bin:literal),* ]) => {
        #[tokio::test]
        async fn creates_global_shims() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let mut plugin = create_plugin($id, sandbox.path());

            plugin.tool.create_shims(false).await.unwrap();

            assert_snapshot!(std::fs::read_to_string(
                sandbox.path().join(".proto/bin").join(if cfg!(windows) {
                    format!("{}.cmd", $id)
                } else {
                    $id.to_string()
                })
            ).unwrap());

            $(
                assert_snapshot!(std::fs::read_to_string(
                    sandbox.path().join(".proto/bin").join(if cfg!(windows) {
                        format!("{}.cmd", $bin)
                    } else {
                        $bin.to_string()
                    })
                ).unwrap());
            )*
        }
    };
}

#[macro_export]
macro_rules! generate_local_shims_test {
    ($id:literal, [ $($bin:literal),* ]) => {
        #[tokio::test]
        async fn creates_global_shims() {
            use proto_core::*;

            let sandbox = create_empty_sandbox();
            let mut plugin = create_plugin($id, sandbox.path());

            plugin.tool.create_shims(false).await.unwrap();

            $(
                assert_snapshot!(std::fs::read_to_string(
                    sandbox.path().join(".proto/tools").join($id).join("latest/shims").join(if cfg!(windows) {
                        format!("{}.ps1", $bin)
                    } else {
                        $bin.to_string()
                    })
                ).unwrap());
            )*
        }
    };
}
