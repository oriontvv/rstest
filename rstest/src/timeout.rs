use std::{sync::mpsc, time::Duration};

#[cfg(feature = "async-timeout")]
use futures::{select, Future, FutureExt};
#[cfg(feature = "async-timeout")]
use futures_timer::Delay;

pub fn execute_with_timeout_sync<T: 'static + Send, F: Fn() -> T + Send + 'static>(
    code: F,
    timeout: Duration,
) -> T {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || sender.send(code()));
    receiver
        .recv_timeout(timeout)
        .unwrap_or_else(|_| panic!("Timeout {:?} expired", timeout))
}

#[cfg(feature = "async-timeout")]
pub async fn execute_with_timeout_async<T, Fut: Future<Output = T>, F: Fn() -> Fut>(
    code: F,
    timeout: Duration,
) -> T {
    select! {
        () = async {
            Delay::new(timeout).await;
        }.fuse() => panic!("Timeout {:?} expired", timeout),
        out = code().fuse() => out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "async-timeout")]
    mod async_version {

        use super::*;
        use std::time::Duration;

        async fn delayed_sum(a: u32, b: u32, delay: Duration) -> u32 {
            async_std::task::sleep(delay).await;
            a + b
        }

        async fn test(delay: Duration) {
            let result = delayed_sum(2, 2, delay).await;
            assert_eq!(result, 4);
        }

        mod use_async_std_runtime {
            use super::*;

            #[async_std::test]
            #[should_panic]
            async fn should_fail() {
                execute_with_timeout_async(
                    || test(Duration::from_millis(20)),
                    Duration::from_millis(10),
                )
                .await
            }

            #[async_std::test]
            async fn should_pass() {
                execute_with_timeout_async(
                    || test(Duration::from_millis(10)),
                    Duration::from_millis(20),
                )
                .await
            }
        }

        mod use_tokio_runtime {
            use super::*;

            #[tokio::test]
            #[should_panic]
            async fn should_fail() {
                execute_with_timeout_async(
                    || test(Duration::from_millis(20)),
                    Duration::from_millis(10),
                )
                .await
            }

            #[tokio::test]
            async fn should_pass() {
                execute_with_timeout_async(
                    || test(Duration::from_millis(10)),
                    Duration::from_millis(20),
                )
                .await
            }
        }
    }

    mod thread_version {
        use super::*;

        pub fn delayed_sum(a: u32, b: u32, delay: Duration) -> u32 {
            std::thread::sleep(delay);
            a + b
        }

        fn test(delay: Duration) {
            let result = delayed_sum(2, 2, delay);
            assert_eq!(result, 4);
        }

        #[test]
        fn should_pass() {
            execute_with_timeout_sync(
                || test(Duration::from_millis(30)),
                Duration::from_millis(70),
            )
        }

        #[test]
        #[should_panic]
        fn should_fail() {
            execute_with_timeout_sync(
                || test(Duration::from_millis(70)),
                Duration::from_millis(30),
            )
        }
    }
}
