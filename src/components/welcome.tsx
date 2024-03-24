import Welcomevideo from "../assets/welcome.mp4"

export const WelcomePage = () => {
  return (
    <div className='h-dvh w-full '>
      <video src={Welcomevideo} className='h-full w-full' autoPlay loop/>
    </div>
  )
}